# gem_grading

## Project Title
gem_grading - On-chain Diamond & Gemstone Grading & Provenance Registry

## Project Description
The global diamond and coloured-gemstone trade still relies on
paper grading reports and hand-written chain-of-custody ledgers.
Reports can be forged, ownership histories are easy to fabricate,
and consumers have no reliable way to confirm that the stone in
front of them matches the certificate in the safe-deposit box.
`gem_grading` solves this by anchoring the lifecycle of every
stone to the Stellar blockchain: miners register raw stones,
certified laboratories attach a tamper-proof grading report, and
every subsequent change of ownership is recorded for provenance.
Because Soroban storage is append-only per-key, the original
grading report is effectively immutable once a lab signs it.

## Project Vision
Our long-term goal is to become a public, trust-minimised registry
that any jeweller, insurer, auction house, or end buyer can query
to verify a stone's 4Cs (carat, color, clarity, cut) and full
chain of custody. By chaining Soroban's near-zero fees and
five-second finality to a network of accredited labs, we aim to
make certified, fraud-resistant provenance as cheap and ubiquitous
as a QR code on a price tag.

## Key Features
- **Stone registration** by the miner with serial, carat weight,
  and country of origin. The miner becomes the initial owner.
- **Immutable grading report** issued by a certified laboratory.
  Once a stone is graded, the color, clarity, cut, and report
  hash can never be overwritten, preserving the integrity of
  the original lab assessment.
- **Provenance tracking** through `transfer_ownership`, which
  records every resale and increments a per-stone transfer
  counter that auditors, insurers, and buyers can verify with
  `verify`.
- **Dispute flagging** by an authorised authority (regulator,
  insurer, or the lab itself). While a dispute is open the
  stone is locked against re-grading and transfer, protecting
  downstream buyers.
- **Lightweight read views** (`get_grade`, `get_owner`,
  `verify`) so wallets and marketplaces can display
  certification status without paying for full state reads.
- **No native asset movement** - prices are recorded as
  metadata only, keeping the contract simple and free of
  custodial risk.

## Contract

- **Network:** Stellar Testnet (Public)
- **Scope:** supply_chain dApp — see `contracts/gem_grading/src/lib.rs` for the full gem_grading business logic.
- **Functions exposed:** see `Key Features` above and the `pub fn` list in `lib.rs`.
- **Contract ID:** CD6SGTRNS6FV4MYWS7WUQHWFPIUYJWCDRQDCYX65TAWMEXKYJQ4NIKSZ
- **Explorer template:** https://stellar.expert/explorer/testnet/tx/4d9b00d2cbb68ae34649d08505b28c4ceca1b6a642f80232e57ff315d2ba702f
- **Screenshot of deployed contract on Stellar Expert:**
  ![screenshot](https://ibb.co/hRBvsqHj)


## Future Scope
- **Multi-lab consensus grading** - require two or more accredited
  labs to agree before a report is finalised, reducing single-lab
  fraud risk for high-value stones.
- **Off-chain report storage with on-chain hash** - store the full
  PDF / image of the lab report on IPFS or another content-addressed
  store, and keep only the hash and a retrieval pointer on chain.
- **Royalty splits for the original miner** - use a payment
  middleware contract to forward a configurable percentage of every
  resale back to the miner who registered the stone.
- **Frontend dApp** - a React/Next.js UI that lets miners, labs,
  dealers, and buyers interact with the contract through Freighter
  and read QR codes printed on physical certificates.
- **Dispute resolution oracle** - integrate a DAO or council
  contract that can lift a `flag_dispute` lock once an investigation
  concludes.
- **Mainnet deployment** with a KYC-gated lab registry so only
  pre-approved addresses can call `grade` on real stones.

## Profile

- **Name:** <!-- Fill github name -->
- **Project:** `gem_grading` (supply_chain)
- **Built with:** Soroban SDK 25, Rust, Stellar Testnet
