# Freighter Wallet Integration Guide

[Freighter](https://www.freighter.app/) is a non-custodial browser extension wallet for the Stellar network.

---

## Installation

```bash
npm install @stellar/freighter-api
```

The `@crabwise/wallets` SDK package already lists this as a dependency, so if
you consume the SDK you do **not** need to install it separately.

---

## Quick Start

```ts
import {
  isFreighterInstalled,
  connectFreighter,
  signTransactionWithFreighter,
  FreighterNotInstalledError,
  NETWORK_PASSPHRASE,
} from "@crabwise/wallets";

// 1. Gate your UI
if (!isFreighterInstalled()) {
  showInstallBanner("https://www.freighter.app/");
}

// 2. Connect
const publicKey = await connectFreighter();
console.log("Connected:", publicKey); // G…

// 3. Sign a transaction
const signedXdr = await signTransactionWithFreighter(
  unsignedXdr,
  NETWORK_PASSPHRASE.TESTNET
);
```

---

## API Reference

### `isFreighterInstalled(): boolean`

Synchronous check. Returns `true` when the Freighter extension is detectable in
the current browser environment.

```ts
if (!isFreighterInstalled()) {
  // prompt the user to install Freighter
}
```

> **Note:** Use this to conditionally render a "Connect Freighter" button. For
> authoritative runtime errors (e.g. the user locked the extension between
> page load and the click), rely on the errors thrown by `connectFreighter()`.

---

### `connectFreighter(): Promise<string>`

Requests access to the user's Stellar account via Freighter. Opens the Freighter
popup on first call; subsequent calls return immediately if the site is already
trusted. Returns the user's public key (`G…`).

```ts
try {
  const publicKey = await connectFreighter();
  // store publicKey in your app state
} catch (err) {
  if (err instanceof FreighterNotInstalledError) {
    window.open(err.downloadUrl, "_blank");
  } else {
    console.error("Connection failed:", err.message);
  }
}
```

**Throws:**
- `FreighterNotInstalledError` — extension is not present in the browser.
- `Error` — user rejected the connection, or Freighter is locked.

---

### `signTransactionWithFreighter(xdr: string, network: string): Promise<string>`

Signs a base64-encoded XDR transaction envelope. Opens the Freighter popup
for user approval. Returns the signed XDR string.

| Parameter | Type     | Description                                    |
|-----------|----------|------------------------------------------------|
| `xdr`     | `string` | Base64-encoded unsigned transaction envelope.  |
| `network` | `string` | Network passphrase (see constants below).      |

```ts
import { NETWORK_PASSPHRASE, signTransactionWithFreighter } from "@crabwise/wallets";
import { TransactionBuilder, Networks, Operation, Asset } from "@stellar/stellar-sdk";

const tx = new TransactionBuilder(sourceAccount, {
  fee: "100",
  networkPassphrase: NETWORK_PASSPHRASE.TESTNET,
})
  .addOperation(/* … */)
  .setTimeout(30)
  .build();

try {
  const signedXdr = await signTransactionWithFreighter(
    tx.toXDR(),
    NETWORK_PASSPHRASE.TESTNET
  );
  // submit signedXdr to Horizon / Soroban RPC
} catch (err) {
  if (err instanceof FreighterNotInstalledError) {
    window.open(err.downloadUrl, "_blank");
  } else {
    console.error("Signing rejected:", err.message);
  }
}
```

**Throws:**
- `FreighterNotInstalledError` — extension is not present in the browser.
- `Error` — user rejected signing or Freighter reported an internal error.

---

### `NETWORK_PASSPHRASE`

```ts
export const NETWORK_PASSPHRASE = {
  MAINNET: "Public Global Stellar Network ; September 2015",
  TESTNET: "Test SDF Network ; September 2015",
} as const;
```

Pass the appropriate constant as the `network` argument to
`signTransactionWithFreighter`.

---

### `FreighterNotInstalledError`

Extends the built-in `Error`. Carries a `downloadUrl` property pointing to
`https://www.freighter.app/` so callers can open the install page directly.

```ts
catch (err) {
  if (err instanceof FreighterNotInstalledError) {
    window.open(err.downloadUrl, "_blank");
  }
}
```

---

## Error Handling Patterns

### Show a download banner

```ts
import { isFreighterInstalled, FREIGHTER_DOWNLOAD_URL } from "@crabwise/wallets";

function WalletConnectButton() {
  if (!isFreighterInstalled()) {
    return (
      <a href={FREIGHTER_DOWNLOAD_URL} target="_blank" rel="noreferrer">
        Install Freighter to continue
      </a>
    );
  }
  return <button onClick={handleConnect}>Connect Freighter</button>;
}
```

### Distinguish rejection from missing extension

```ts
import { connectFreighter, FreighterNotInstalledError } from "@crabwise/wallets";

async function handleConnect() {
  try {
    const key = await connectFreighter();
    setPublicKey(key);
  } catch (err) {
    if (err instanceof FreighterNotInstalledError) {
      setError("Please install the Freighter extension.");
      window.open(err.downloadUrl, "_blank");
    } else {
      setError("Connection was cancelled or Freighter is locked.");
    }
  }
}
```

---

## End-to-End Example: Donate via crabwise

```ts
import {
  connectFreighter,
  signTransactionWithFreighter,
  NETWORK_PASSPHRASE,
} from "@crabwise/wallets";
import { Server, TransactionBuilder, Networks, Operation, Asset } from "@stellar/stellar-sdk";

const HORIZON_URL = "https://horizon-testnet.stellar.org";
const DONATION_CONTRACT = "<campaign-contract-id>";

async function donate(amountXlm: string) {
  // 1. Connect wallet
  const publicKey = await connectFreighter();

  // 2. Build transaction
  const server = new Server(HORIZON_URL);
  const account = await server.loadAccount(publicKey);
  const tx = new TransactionBuilder(account, {
    fee: "1000",
    networkPassphrase: NETWORK_PASSPHRASE.TESTNET,
  })
    .addOperation(
      Operation.payment({
        destination: DONATION_CONTRACT,
        asset: Asset.native(),
        amount: amountXlm,
      })
    )
    .setTimeout(60)
    .build();

  // 3. Sign
  const signedXdr = await signTransactionWithFreighter(
    tx.toXDR(),
    NETWORK_PASSPHRASE.TESTNET
  );

  // 4. Submit
  const result = await server.submitTransaction(
    TransactionBuilder.fromXDR(signedXdr, NETWORK_PASSPHRASE.TESTNET)
  );
  console.log("Transaction hash:", result.hash);
}
```

---

## Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| `FreighterNotInstalledError` | Extension not installed or not enabled for this origin | Direct user to `https://www.freighter.app/` |
| "Freighter access denied" | User clicked Reject in the popup | Ask the user to try again and click Accept |
| `signTransaction` fails with no error | Extension is locked | Ask the user to unlock Freighter and retry |
| `isFreighterInstalled()` returns `false` in a test environment | `window` or `window.freighter` is not defined | Mock `window.freighter` in your test setup |
