# IronCalc service application

This directory contains the code (frontend and backend) to run the code deployed at https://app.ironcalc.com

## Development build:

1. Run in this folder `caddy run` (that just just a proxy for the front end and backend).
   You will need to leave it running all the time.
2. In the server folder run `cargo run`
3. In the frontend folder `npm install` and `npm run dev`

That's it if you point your browser to localhost:2080 you should see the app.

Note that step three involves alo building thw wasm bindings and the widget

## Deployment

The development environment is very close to a deployment environment.

### Build the server binary:

In the server directory run:

```
cargo build --release
```

You will find a single binary in target/release/ironcalc_server

### Build the frontend files

In the frontend folder:

```
npm install
npm run build
```

That will create a bunch of files that you should copy to your server

## TODO

Deployment details, brotli, logs, stats, Postgres, systemctl files, ...

