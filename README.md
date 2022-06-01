# Hex
[Hex](https://en.wikipedia.org/wiki/Hex_(Discworld)) is a solver for the Directed Feedback Vertex Set Problem. The problem states that given a graph $G = (V, E)$, we
compute a minimum-sized subset $X \subseteq V$ such that the induced graph of $G[V \setminus X]$ is acyclic. This solver was submitted to the PACE Challange 2022 and is written for my master thesis. 


## Requirements
The project relies on interfacing with the [Coin-or CBC library](https://github.com/coin-or/COIN-OR-OptimizationSuite)
* [Coin-or cbc library](https://github.com/coin-or/COIN-OR-OptimizationSuite), which can be installed with apt using 
    
    ```$ apt-get install coinor-libcbc-dev```.

Furthermore, this project uses the Vertex Cover solver of Hespe et al., the winners of the PACE 2019 Vertex Cover (exact) challange. Their repository is available [here](https://github.com/KarlsruheMIS/pace-2019). We use the binary with the `pace-2019` tag, which is already shipped inside `extern/WeGotYouCovered/vc_solver`.

Other main dependencies are fetched and compiled using cargo. These dependencies are `assert_cmd`, `clap` and `coin_cbc`, `rand`, `rustc_hash`, and `shh` (to force cbc to be quiet).

For development purposes, cargo also installs `cpu_time`, and `rayon`. These dependencies are not strictly necessary for the solver. 

## Building
The solver can be built using `cargo build --release`. The resulting binary is located in `/target/release/hex`. In order to run the binary from any directory, note that the `vc_solver` from WeGotYouCovered must be placed inside the directory `extern/WeGotYouCovered/vc_solver`, i.e., relative from you *current working directory*. You may need to set execution rights on `vc_solver` manually after cloning from GitHub. 

Alternatively, the binary can be compiled with the feature `root-vc-solver`, i.e., compiling with `cargo build --release --features root-vc-solver`, `hex` will search for `vc_solver` in the root of the current working directory. 

## Installation
Hex can be installed in your PATH using `cargo install --path . --bin hex`, and can be uninstalled with `cargo uninstall --bin hex`.

## Arguments
Hex has only one argument, `time-limit-vc`, it sets the time limit to run the Vertex Cover solver of Hespe et al., with the default at 300 seconds.

## Optil
Unfortunately, the optil.io server is running on Ubuntu 16.04 (last checked in April 2022), which comes with the caveat that building the project on your current favorite linux distro compiles and links the binaries against newer versions of `glibc`. To remedy this, we use a docker container that is running Ubuntu 16.04, we build the binary in that environment, and extract it. This guarantees we obtain a working binary for optil, since it is compiled and linked in the same environment.

### Requirements
* [Podman](https://podman.io/getting-started/installation)

### Building
The binary can be built using 
```
$ ./build_optil.sh
```
It will create the directory `optil_target/` which contains both the binary and a tarball which can be submitted directly to the platform. 

### Remark About Clap
Optil.io was not able to run the binary when `clap` was included, so the binary was built without `clap`. However, since `clap` is great for setting optional flags, the release is built with `clap`. 
