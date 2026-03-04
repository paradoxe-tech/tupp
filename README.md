# tupp

A minimalist command-line contact manager. Contacts and groups are stored locally as JSON and can optionally be exposed over a simple HTTP API.

## Easy Install

To install `tupp` with ease, you can use its version manager: `tuppdate` is a shell script that downloads the right release binary for your system from GitHub and installs it in `$HOME/.local/bin`. Before installing, please make sure it's in your `PATH`.

```bash
curl -sL https://raw.githubusercontent.com/mtripnaux/tupp/refs/heads/main/tuppdate.sh -o tuppdate.sh
chmod +x tuppdate.sh
mkdir -p `"`$HOME/.local/bin`"
mv ./tuppdate.sh `"`$HOME/.local/bin/tuppdate`"
```

Since then, you should be able to run this from anywhere:

```bash
tuppdate latest       # install the latest release
tuppdate 1.3.1        # install a specific version
```

## Build from source

Requires the [Rust](https://rustup.rs/) package manager, Cargo.

```bash
git clone https://github.com/mtripnaux/tupp.git
cd tupp
cargo build --release
cargo install --path .
```

## Usage

```bash
tupp --help
```

## Tupp Server

One could want its personal contacts to be accessible from outside its local network. For example, multiple web applications use `tupp` as an engine, accessed from a visual and user-friendly graphical interface. To se up your server and expose your contacts to the outside world, please create a password (here referenced as a secret, or token) and run the following command: 

```bash
TUPP_API_TOKEN=<secret> tupp serve --port 8080
```

The `8080` port will be opened, and you will be able to access your tupp server at `<your-ip>:8080` whenever you want.

| Method | Path        | Description                                        |
|--------|-------------|----------------------------------------------------|
| GET    | `/contacts` | Return your contacts list as JSON                  |
| POST   | `/contacts` | Create (no `identifier`) or update a contact       |

However, all requests require a Bearer token header with your super-secret token, preventing the pirates from stealing your personal data.

```bash
Authorization: Bearer <secret>
```