# Auto-campnet GUI

### What is it?

A small cross platform application that manages logging in to BITS Goa campus network automatically

### How do I install it?

Download the latest build from [releases page](https://github.com/riskycase/auto-campnet-gui/releases)

## Roadmap

- [x] Build app UI
- [ ] Set up login backend
- [ ] Save user credentials
- [ ] Allow user to add app to startup from within the app
- [ ] Tray icon and actions
- [ ] Query Sophos user dashboard to show remaining data for the day

## Contributing

### Set up development environment

Follow the instructions given in Tauri's [prerequisites](https://tauri.studio/docs/getting-started/prerequisites) page to set up your machine according to the OS.

After that, in cloned repository run
```sh
npm i
```
to install the app dependencies

### Running the app in dev mode

Use following command to run the app in development mode. Tauri's development mode allows the app to monitor the rust backend for file changes and reloads the app automatically.
```sh
npm run dev
```

### Building the app

Use following command to build the app
```sh
npm run build
```
