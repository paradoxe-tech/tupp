## Installation

### Quick install using `tuppdate`

To easily manage updates for the tupp binary, we provide a self-contained script called tuppdate. It downloads the specific release for your system and installs it in your local directory (`$HOME/.local/bin`). For example, you can run `tuppdate 1.1.0`.

```bash
curl -sL https://gist.github.com/paradoxe-tech/369ff4ca0acdff4b2b4424a69ca52bc9/raw/tuppdate.sh -o tuppdate.sh
chmod +x tuppdate.sh
mkdir -p "$HOME/.local/bin"
mv ./tuppdate.sh "$HOME/.local/bin/tuppdate"
```
```bash
tuppdate 1.1.0
```

### Compile it yourself

You must have Rust and Cargo installed on your system. 

```bash
git clone https://github.com/paradoxe-tech/tupp.git
cd tupp
cargo build --release
cargo install --path
```

## Usage

```bash
tupp --help
```

## Data Storage

Contact data is stored in `~/.config/tupp/contacts.json` on Linux/macOS and `%APPDATA%\tupp\contacts.json` on Windows.
