# 💸 Solana TipJar

**TipJar** is a decentralized tipping platform built on the **Solana blockchain** using the **Anchor framework**. It allows users to:

- Create personalized tip jars  
- Accept SOL tips with optional memos and visibility options  
- Set and track tipping goals  
- Manage tip histories  
- Withdraw funds  
- Pause, resume, or close tip jars  

All actions are stored **on-chain** for full transparency and decentralization.

---

## 🛠️ Features

- 🧠 **Initialize TipJar** – Create a unique tip jar with description, category, and goal.  
- 🎁 **Send Tips** – Anyone can tip a user with SOL, along with a public or private message.  
- 📊 **Track Stats** – Get on-chain insights into tip count and total received SOL.  
- 🗂️ **Manage History** – Clear tip history while keeping funds.  
- 🚦 **Control State** – Pause/resume/close your tip jar anytime.  
- 📝 **Update Metadata** – Change your tip jar’s details anytime.  

---

## 📦 Program Overview

### Program Accounts

**TipJar Account Fields:**

- `owner`: Creator of the tip jar  
- `description`: Purpose of the tip jar  
- `category`: E.g., Education, Art, Development  
- `goal`: SOL target (optional)  
- `total_received`: Total tips received  
- `tips_history`: List of `Tip` structs  
- `tip_count`: Total number of tips  
- `is_active`: Whether the tip jar is accepting tips  

### Tip Struct

```rust
pub struct Tip {
    pub sender: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
    pub visibility: TipVisibility,
    pub memo: String,
}
```

### TipVisibility Enum

```rust
pub enum TipVisibility {
    Public,
    Private,
}
```

## 🚀 Getting Started

### 1. Install Prerequisites:
- Solana CLI
- Anchor CLI
- Node.js and NPM/Yarn

### 2. Clone the Repo

```bash
git clone https://github.com/your-username/solana-tipjar.git
cd solana-tipjar
```

### 3. Build and Deploy

```bash
anchor build
anchor deploy
```

Set your cluster using anchor test --provider.cluster devnet or by updating Anchor.toml.

## 🧪 Testing

Run all test cases using:

```bash
anchor test
```

This runs a comprehensive suite that tests:

- Initialization
- Tipping
- Updating metadata
- Clearing history
- Toggling state
- Withdrawing funds
- Closing the tip jar

## 🧩 Program Instructions

### Initialize TipJar

```ts
initializeTipjar(description: string, category: string, goal: BN)
```

### Send Tip

```ts
sendTip(amount: BN, visibility: TipVisibility, memo: string)
```

### Get Tip Stats

```ts
getTipStats()
```

### Clear Tip History

```ts
clearTipHistory()
```

### Toggle Status (Pause/Resume)

```ts
toggleTipjarStatus()
pauseTipjar()
resumeTipjar()
```

### Update TipJar Info

```ts
updateTipjar(description: string, category: string, goal: BN)
```

### Withdraw Tips

```ts
withdrawTip(amount: BN)
```

### Close TipJar

```ts
closeTipjar()
```

## 📁 Directory Structure

```bash
solana-tipjar/
├── programs/
│   └── solana-tipjar/
│       └── src/lib.rs  
        └── src/state.rs      # Main Anchor program logic
├── tests/
│   └── solana-tipjar.ts       # Anchor Mocha tests
├── migrations/
├── Anchor.toml
├── Cargo.toml
└── README.md
```

## 🧑‍💻 Contributing

Pull requests are welcome! For major changes, open an issue first to discuss what you’d like to change or add.

## 📄 License

MIT License

## 🌐 Live Demo (Optional)
You can connect this program to a frontend using React/Next.js + Solana Wallet Adapter + Anchor Client.

Example frontend coming soon…

## 👋 Connect
Made with 💙 by Aditya Mishra