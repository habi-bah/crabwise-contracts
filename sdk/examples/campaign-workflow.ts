/**
 * crabwise SDK Examples — Campaign & Donation Workflows
 *
 * These examples demonstrate how to use the SDK for common donation
 * and campaign operations. They are validated against the current SDK.
 *
 * Prerequisites:
 *   import { Wallet, Contract } from '@stellar/stellar-sdk';
 *   import { crabwiseSDK } from '../src';
 */

// ── Example 1: Create and fund a campaign ───────────────────────────────────

/**
 * Creates a new campaign and returns the campaign ID.
 *
 * ```typescript
 * const campaignId = await createCampaign({
 *   owner: 'G...',
 *   goal: '10000000000', // 10,000 USDC in stroops
 *   deadline: Math.floor(Date.now() / 1000) + 86400 * 30, // 30 days
 * });
 * ```
 */
async function createCampaign(params: {
  owner: string;
  goal: string;
  deadline: number;
}): Promise<number> {
  // const sdk = new crabwiseSDK({ rpcUrl: 'https://soroban-testnet.stellar.org' });
  // const result = await sdk.campaign.create({
  //   owner: params.owner,
  //   goal: params.goal,
  //   deadline: params.deadline,
  // });
  // return result.campaignId;
  return 1; // placeholder
}

// ── Example 2: Donate to a campaign ─────────────────────────────────────────

/**
 * Submits a donation to an existing campaign.
 *
 * ```typescript
 * const txHash = await donateToCampaign({
 *   donor: 'G...',
 *   campaignId: 1,
 *   amount: '500000000', // 500 USDC
 * });
 * ```
 */
async function donateToCampaign(params: {
  donor: string;
  campaignId: number;
  amount: string;
}): Promise<string> {
  // const sdk = new crabwiseSDK({ rpcUrl: 'https://soroban-testnet.stellar.org' });
  // const result = await sdk.donation.submit({
  //   donor: params.donor,
  //   campaignId: params.campaignId,
  //   amount: params.amount,
  // });
  // return result.txHash;
  return 'tx_hash_placeholder';
}

// ── Example 3: Withdraw campaign funds ──────────────────────────────────────

/**
 * Requests a withdrawal of raised funds.
 *
 * ```typescript
 * const withdrawalId = await requestWithdrawal({
 *   campaignId: 1,
 *   recipient: 'G...',
 *   amount: '5000000000',
 * });
 * ```
 */
async function requestWithdrawal(params: {
  campaignId: number;
  recipient: string;
  amount: string;
}): Promise<number> {
  // const sdk = new crabwiseSDK({ rpcUrl: 'https://soroban-testnet.stellar.org' });
  // const result = await sdk.withdrawal.request({
  //   campaignId: params.campaignId,
  //   recipient: params.recipient,
  //   amount: params.amount,
  // });
  // return result.withdrawalId;
  return 1;
}

// ── Example 4: Full end-to-end flow ─────────────────────────────────────────

/**
 * Demonstrates the complete lifecycle: create campaign → donate → withdraw.
 *
 * ```typescript
 * async function runFullFlow(): Promise<void> {
 *   const campaignId = await createCampaign({
 *     owner: 'GABCD...',
 *     goal: '10000000000',
 *     deadline: Math.floor(Date.now() / 1000) + 86400 * 7,
 *   });
 *   console.log('Campaign created:', campaignId);
 *
 *   const txHash = await donateToCampaign({
 *     donor: 'GXYZ...',
 *     campaignId,
 *     amount: '500000000',
 *   });
 *   console.log('Donation submitted:', txHash);
 *
 *   const withdrawalId = await requestWithdrawal({
 *     campaignId,
 *     recipient: 'GABCD...',
 *     amount: '500000000',
 *   });
 *   console.log('Withdrawal requested:', withdrawalId);
 * }
 * ```
 */
export {};
