# DFVStritus
The DFVStritus framework is a solver for the Directed Feedback Vertex Set Problem. The problem states that given a graph G = (V, E), we
compute a minimum-sized subset X of V such that the induced graph of G[V \ X] is acyclic. This solver was submitted to the PACE Challange 2022 and is written for my master thesis. 

## Installation

## Requirements
The project relies on interfacing with the [Coin-or CBC library](https://github.com/coin-or/COIN-OR-OptimizationSuite)
* [Coin-or cbc library](https://github.com/coin-or/COIN-OR-OptimizationSuite), which can be installed with apt using 
    
    ```$ apt-get install coinor-libcbc-dev```.

Furthermore, this project uses the Vertex Cover solver of WeGotYouCovered, the winners of the PACE 2019 Vertex Cover (exact) challange. Their repository is available [here](https://github.com/KarlsruheMIS/pace-2019). We use the binary with the `pace-2019` tag, which is already shipped inside `extern/WeGotYouCovered/vc_solver`.

Other main dependencies are fetched and compiled using cargo. These dependencies are `rustc-hash`, `rand`, `duct`, `shh` (to force cbc to be quiet), and `coin_cbc` for the rust bindings to cbc.

For development purposes, cargo also installs `cpu_time`, `rayon` and `assert_cmd`. These dependencies are not strictly necessary for the solver.

## Building
### Linux
The solver can be built using `cargo build --release`. The resulting binary is located in `/target/release/dfvstritus`. In order to run the binary from any directory, note that the `vc_solver` from WeGotYouCovered must be placed inside the directory `extern/WeGotYouCovered/vc_solver`, i.e., relative from you *current working directory*. You may need to set execution rights on `vc_solver` manually after cloning from GitHub. 

# Optil
Unfortunately, the optil.io server is running on Ubuntu 16.04 (last checked in April 2022), which comes with the caveat that building the project on your current favorite linux distro compiles and links the binaries against newer versions of `glibc` and `coinor-libcbc-dev`. To remedy this, we use a docker container that is running Ubuntu 16.04, we build the binary in that environment, and extract it. This ensures the binary is linked to the correct versions of `glibc` and `coinor-libcbc-dev` without the need of creating a VM.

### Requirements
* [Podman](https://podman.io/getting-started/installation)

### Building
The binary can be built using 
```
$ ./build_optil.sh
```
It will create the directory `optil_target/` which contains both the binary and a tarball which can be submitted directly to the platform. 