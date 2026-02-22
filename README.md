# Tupp

## Installation

### Quick install using `tuppdate`

To easily manage updates for the tupp binary, we provide a self-contained script called tuppdate. It downloads the specific release for your system and installs it in your local directory (`$HOME/.local/bin`). For example, you can run `tuppdate 1.2.2`.

```bash
curl -sL https://raw.githubusercontent.com/mtripnaux/tupp/refs/heads/main/tuppdate.sh -o tuppdate.sh
chmod +x tuppdate.sh
mkdir -p "$HOME/.local/bin"
mv ./tuppdate.sh "$HOME/.local/bin/tuppdate"
```

```bash
tuppdate 1.2.2
```

### Compile it yourself

You must have Rust and Cargo installed on your system.

```bash
git clone https://github.com/mtripnaux/tupp.git
cd tupp
cargo build --release
cargo install --path
```

## Data Storage

Contact data is stored in `~/.config/tupp/contacts.json` on Linux/macOS and `%APPDATA%\tupp\contacts.json` on Windows.
