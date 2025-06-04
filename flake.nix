{
  description = "Flake with Rust and Go projects";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ rust-overlay.outputs.overlays.default ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default;

        rustPackage = pkgs.rustPlatform.buildRustPackage {
          pname = "storyteller-ui";
          version = "0.1.0";
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
        };

        goPackage = pkgs.buildGoModule {
          pname = "storyteller-api";
          version = "0.1.0";
          src = ./api; 
          vendorHash = "sha256-YduyX9vfYX7cVVotCCxXnX4uYuaxVs5IpMFaRChBfdA=";
        };
      in {
        packages.default = pkgs.symlinkJoin {
          name = "storyteller";
          paths = [ rustPackage goPackage ];
        };

        devShells.default = pkgs.mkShell {
          buildInputs = [
            rustToolchain
            pkgs.go
            pkgs.pkg-config
            pkgs.openssl
            pkgs.gorm-gentool
          ];

          shellHook = ''
            echo "Rust and Go dev environment loaded."
          '';
        };
      }
    );
}