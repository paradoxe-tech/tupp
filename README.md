
## Installation

### Prerequisites

You need to have Rust installed on your system. If you don't have Rust installed:

**Linux/macOS:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

**Windows:**
Download and run the installer from [rustup.rs](https://rustup.rs/)

### Installing Tupp

1. **Clone the repository:**
   ```bash
   git clone https://github.com/paradoxe-tech/tupp.git
   cd tupp
   ```

2. **Build and install:**
   ```bash
   cargo build --release
   ```

3. **Run the application:**

   ```bash
   cargo install --path .
   tupp --help
   ```

## Features

- **Contact Management**: Create, update, and manage contacts
- **Gender Support**: Mandatory gender selection (Male, Female, Non-binary)
- **Birth/Death Information**: Track birth dates, locations, and death information
- **Relationships**: Link contacts with family and professional relationships
- **Search**: Find contacts by name or other details
- **Export**: Export contact data to JSON

## Data Storage

Contact data is stored in `~/.config/tupp/contacts.json` on Linux/macOS and `%APPDATA%\tupp\contacts.json` on Windows.
