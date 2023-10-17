import {
  AttestationQueueAccount,
  FunctionAccount,
  FunctionRequestAccount,
  SwitchboardProgram,
  attestationTypes,
  loadKeypair,
} from "@switchboard-xyz/solana.js";
import * as anchor from "@coral-xyz/anchor";
import { SuperiorRandomness } from "../target/types/superior_randomness";
import dotenv from "dotenv";
import chalk from "chalk";
import {
  addMrEnclave,
  loadSwitchboardFunctionEnv,
  myMrEnclave,
  loadDefaultQueue,
  CHECK_ICON,
  FAILED_ICON,
} from "./utils";
import { TestMeter } from "./meter";
const nacl = require("tweetnacl");
import { createHash } from "crypto";
import {
  Ed25519Program,
  SYSVAR_INSTRUCTIONS_PUBKEY,
  sendAndConfirmTransaction,
  Transaction,
} from "@solana/web3.js";

dotenv.config();

interface RequestSeededEvent {
  request: anchor.web3.PublicKey;
  seed: number;
}

interface CostReceipt {
  name: string;
  description?: string;
  cost: number;
}

(async () => {
  if (!myMrEnclave) {
    throw new Error(
      `You need a ./measurement.txt in the project root or define MR_ENCLAVE in your .env file`
    );
  }

  console.log(
    `\n${chalk.green(
      "This script will invoke our VRF program and request a random value. If SWITCHBOARD_FUNCTION_PUBKEY is not present in your .env file then a new Switchboard Function will be created each time. A Switchboard Request is an instance of a function and allows us to pass custom parameters. In this example we create a new request each time - this is wasteful and requires the user to manually close these after use. See the Switchboard Randomness Callback example program for a more complete implementation where each user has a dedicated request account."
    )}`
  );

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(
    process.argv.length > 2
      ? new anchor.AnchorProvider(
          provider.connection,
          new anchor.Wallet(loadKeypair(process.argv[2])),
          {}
        )
      : provider
  );

  const program: anchor.Program<SuperiorRandomness> =
    anchor.workspace.SuperiorRandomness;

  const payer = (provider.wallet as anchor.Wallet).payer;
  console.log(`[env] PAYER: ${payer.publicKey}`);

  const testMeter = new TestMeter(program, "simple-flip", {
    useSolUnits: true,
  });

  const switchboardProgram = await SwitchboardProgram.fromProvider(provider);

  let switchboardFunction: FunctionAccount | undefined = undefined;
  let functionState: attestationTypes.FunctionAccountData | undefined =
    undefined;

  let attestationQueue: AttestationQueueAccount | undefined = undefined;

  [switchboardFunction, functionState] = await loadSwitchboardFunctionEnv(
    switchboardProgram
  );

  // If we loaded our function from .env, try to add the measurement.txt to our function config
  if (switchboardFunction) {
    attestationQueue = new AttestationQueueAccount(
      switchboardProgram,
      functionState.attestationQueue
    );

    const addEnclaveTxn = await addMrEnclave(
      switchboardProgram,
      switchboardFunction,
      functionState
    );

    if (addEnclaveTxn) {
      console.log(
        `[TX] function_set_config (added MrEnclave): ${addEnclaveTxn}`
      );
    }
  } else {
    // Create a new Switchboard Function
    if (!process.env.DOCKER_IMAGE_NAME) {
      throw new Error(
        `You need to set DOCKER_IMAGE_NAME in your .env file to create a new Switchboard Function. Example:\n\tDOCKER_IMAGE_NAME=gallynaut/solana-simple-randomness-function`
      );
    }

    // Get the Attestation queue address from the clusters genesis hash
    attestationQueue = await loadDefaultQueue(switchboardProgram);

    const { data: functionInitTx, receipt: functionInitReceipt } =
      await testMeter.run("function_init", async (meter) => {
        let tx: string;
        [switchboardFunction, tx] = await FunctionAccount.create(
          switchboardProgram,
          {
            name: "commit-reveal-randomness",
            metadata:
              "https://github.com/switchboard-xyz/solana-simple-randomness-example/tree/main/switchboard-function",
            container: process.env.DOCKER_IMAGE_NAME,
            containerRegistry: "dockerhub",
            version: "latest",
            attestationQueue,
            authority: payer.publicKey,
            mrEnclave: myMrEnclave,
          }
        );

        console.log(
          `\nMake sure to add the following to your .env file:\n\tSWITCHBOARD_FUNCTION_PUBKEY=${switchboardFunction.publicKey}\n\n`
        );

        meter.addReceipt({
          name: "function_init",
          tx: tx,
          rent: [
            {
              account: "FunctionAccount",
              description: "SwitchboardFunction account",
              cost:
                (await program.provider.connection.getMinimumBalanceForRentExemption(
                  switchboardProgram.attestationAccount.functionAccountData.size
                )) / anchor.web3.LAMPORTS_PER_SOL,
            },
            {
              account: "SwitchboardWallet",
              description: "Re-useable wallet for Switchboard functions",
              cost:
                (await program.provider.connection.getMinimumBalanceForRentExemption(
                  switchboardProgram.attestationAccount.switchboardWallet.size
                )) / anchor.web3.LAMPORTS_PER_SOL,
            },
            {
              account: "SwitchboardWallet Escrow",
              description: "Wrapped SOL token account used to pay for requests",
              cost:
                (await program.provider.connection.getMinimumBalanceForRentExemption(
                  165 /** TokenAccount bytes */
                )) / anchor.web3.LAMPORTS_PER_SOL,
            },
            {
              account: "AddressLookupTable",
              description:
                "Solana Address Lookup table to support versioned transactions in the future",
              cost:
                (await program.provider.connection.getMinimumBalanceForRentExemption(
                  568 /** Address Lookup Account bytes */
                )) / anchor.web3.LAMPORTS_PER_SOL,
            },
          ],
        });

        return tx;
      });

    // Log the txn signatures and add the cost receipts for easier debugging
    console.log(`[TX] function_init: ${functionInitTx}`);
  }

  const attestationQueueState = await attestationQueue.loadData();

  /////////////////////////////////
  ////      RANDOMNESS         ////
  /////////////////////////////////

  const requestPubkey = anchor.web3.Keypair.generate().publicKey;
  const seed = createHash("sha256").update(requestPubkey.toBytes()).digest();
  const [pda] = await anchor.web3.PublicKey.findProgramAddress(
    [seed],
    program.programId
  );
  const switchboardRequestKeypair = anchor.web3.Keypair.generate();
  const switchboardRequestEscrowPubkey = anchor.utils.token.associatedAddress({
    mint: switchboardProgram.mint.address,
    owner: switchboardRequestKeypair.publicKey,
  });

  const switchboardRequest = new FunctionRequestAccount(
    switchboardProgram,
    switchboardRequestKeypair.publicKey
  );
  console.log(`Request account: ${switchboardRequestKeypair.publicKey}`);

  const {
    data: [event, slot],
    receipt: guessReceipt,
  } = await testMeter.runAndAwaitEvent(
    "request",
    "RequestSeededEvent",
    (event) => event.request.equals(pda),
    async (meter) => {
      const tx = await program.methods
        .request(seed)
        .accounts({
          payer: payer.publicKey,
          req: pda,
          authority: payer.publicKey,
          switchboard: switchboardProgram.attestationProgramId,
          switchboardState:
            switchboardProgram.attestationProgramState.publicKey,
          switchboardAttestationQueue: attestationQueue.publicKey,
          switchboardFunction: switchboardFunction.publicKey,
          switchboardRequest: switchboardRequest.publicKey,
          switchboardRequestEscrow: switchboardRequestEscrowPubkey,
          switchboardMint: switchboardProgram.mint.address,
        })
        .signers([switchboardRequestKeypair])
        .rpc();

      meter.addReceipt({
        name: "request",
        tx: tx,
        sbFee: attestationQueueState.reward,
        rent: [
          {
            account: "FunctionRequest",
            description:
              "Rent exemption for the Switchboard FunctionRequestAccount",
            cost:
              (await program.provider.connection.getMinimumBalanceForRentExemption(
                await program.provider.connection
                  .getAccountInfo(switchboardRequest.publicKey)
                  .then((a) => a?.data.length)
                  .catch(
                    () =>
                      switchboardProgram.attestationAccount
                        .functionRequestAccountData.size + 512
                  )
              )) / anchor.web3.LAMPORTS_PER_SOL,
          },
          {
            account: "FunctionRequest Escrow",
            description: "Wrapped SOL token account used to pay for requests",
            cost:
              (await program.provider.connection.getMinimumBalanceForRentExemption(
                165 /** TokenAccount bytes */
              )) / anchor.web3.LAMPORTS_PER_SOL,
          },
          ...[
            {
              account: "UserState",
              cost:
                (await program.provider.connection.getMinimumBalanceForRentExemption(
                  program.account.requestAccountData.size
                )) / anchor.web3.LAMPORTS_PER_SOL,
            },
          ],
          ,
        ],
      });
    }
  );

  console.log(`\n${CHECK_ICON} Request seed planted!! ${event.seed}\n`);

  const msg = numberToLittleEndianBuffer(event.seed);

  const tx = new Transaction();
  const ix = await program.methods
    .reveal(requestPubkey)
    .accounts({
      req: pda,
    })
    .instruction();
  tx.add(ix);
  tx.recentBlockhash = (
    await program.provider.connection.getRecentBlockhash()
  ).blockhash;
  tx.sign(payer);
  const sig = await sendAndConfirmTransaction(program.provider.connection, tx, [
    payer,
  ]);

  await testMeter.stop();

  console.log(`\n### METRICS`);

  const fullDuration = guessReceipt.time.delta;
  console.log(`Settlement Time: ${fullDuration.toFixed(3)} seconds`);

  console.log(
    `Cost: ${guessReceipt.balance.delta} ${
      testMeter.config.balance.units === "sol" ? "SOL" : "lamports"
    }`
  );

  testMeter.print();
  console.log(`Randomness revelead on chain: ${sig}`);
})();

function numberToLittleEndianBuffer(value, bytes = 4) {
  const buffer = new ArrayBuffer(bytes);
  const view = new DataView(buffer);

  // Assuming it's a 4-byte (32-bit) integer by default
  // You can modify to support 2-byte, 8-byte, etc. as needed
  switch (bytes) {
    case 2:
      view.setInt16(0, value, true); // true for little-endian
      break;
    case 4:
      view.setInt32(0, value, true);
      break;
    case 8:
      view.setBigInt64(0, BigInt(value), true);
      break;
    default:
      throw new Error("Unsupported byte length");
  }

  return new Uint8Array(buffer);
}
