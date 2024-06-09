{
  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, fenix }:
  let
    # TODO: Support more architectures
    system = "x86_64-linux";
    overlays = [ fenix.overlays.default ];
    
    pkgs = import nixpkgs {
      inherit system overlays;
    };
  in
  {
    devShells.${system}.default = pkgs.callPackage ./shell.nix {};
  };
}
