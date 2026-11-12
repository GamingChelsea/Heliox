# Heliox

Heliox is a Rust-only NSH app. Run it from the project directory:

```sh
nsh run
```

Useful NSH commands:

```sh
nsh check
nsh test
nsh build
nsh package --target windows
```

`nsh run` uses the platform's native development host by default. Choose another host with `--target`, for example
`nsh run --target gpui`. NSH-generated files are kept in `.nsh/`.

For the standalone native runner, use `cargo run`.
