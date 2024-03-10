{ mkShell, pkgsCross, fenix }:

# Reference: https://github.com/nix-community/naersk/blob/aeb58d5e8faead8980a807c840232697982d47b9/examples/cross-windows/flake.nix

let
  channel = "stable";

  targets = with fenix.targets; [
    x86_64-unknown-linux-gnu
  ];

  crossCompileTargets = with fenix.targets; [
    x86_64-pc-windows-gnu
  ];
  
  toolchain = fenix.combine (with fenix.${channel}; [
    cargo
    rustc
  ] ++ map (target: target.${channel}.toolchain) targets
    ++ map (target: target.${channel}.rust-std) crossCompileTargets);

in

mkShell {
  buildInputs = with pkgsCross.mingwW64; [
    stdenv.cc
  ];

  packages = [ toolchain ];
  
  # Link to libpthreads manually otherwise build fails
  # See: https://github.com/NixOS/nixpkgs/issues/139966#issuecomment-1385222547
  env.CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUSTFLAGS = "-L native=${pkgsCross.mingwW64.windows.pthreads}/lib";
}
