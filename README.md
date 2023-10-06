# PXU gui

PXU gui is a companion to the paper arXiv:XXXX.XXXXX. It gives a visualisation of the relation between the variables p, x⁺, x⁻ and u which are useful for describing the kinematics of world-sheet excitations of the light-cone gauge string in AdS₃ × S³ × T⁴ supported by a mix of RR and NSNS flux.

The easiest way to run PXU gui is to simply go to the [web site](https://olofos.github.io/pxu-gui/).

PXU gui can also be run as a native application. It is built using rust. The graphical interface is uses [egui](https://github.com/emilk/egui/) and [eframe](https://github.com/emilk/egui/tree/master/crates/eframe).

### Running locally as a native application

On Ubuntu and related Linux distribution you can install all needed dependencies using

`sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libspeechd-dev libxkbcommon-dev libssl-dev`

After cloning the repository simply run

`cargo run --bin pxu-gui --release`

to build and run the application.


### Running the web version locally

The web version of PXU gui works by be compiling the code to [WASM](https://en.wikipedia.org/wiki/WebAssembly). It uses [Trunk](https://trunkrs.dev/) to build for web target.

1. Install Trunk with `cargo install --locked trunk`.
2. Run `trunk serve pxu-gui/index.html --release` to build and serve on `http://127.0.0.1:8080`. Trunk will rebuild automatically if you edit the project.
3. Open `http://127.0.0.1:8080/index.html` in a browser.

## License

PXU gui is licensed under the [MIT license](https://github.com/olofos/pxu-gui/blob/master/LICENSE).
