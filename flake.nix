{
  description = "Combined flake with Rust/Go projects and static Graphviz build";

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

        # Rust and Go packages

        rustToolchain = pkgs.rust-bin.stable.latest.default;

        rustPackage = pkgs.rustPlatform.buildRustPackage {
          pname = "storyteller-ui";
          version = "0.1.0";
          src = ./.;
          nativeBuildInputs = [
            staticGraphviz
            pkgs.pkg-config
            pkgs.libclang
            pkgs.clang
            pkgs.zlib
          ];
          buildInputs = [
            staticGraphviz
            pkgs.pkg-config
          ];

          LIBCLANG_PATH="${pkgs.llvmPackages.libclang.lib}/lib";
          INCLUDE_DIR="${staticGraphviz}/include/";
          PKG_CONFIG_PATH="${staticGraphviz}/lib/pkgconfig";
          BINDGEN_EXTRA_CLANG_ARGS = ''$(pkg-config --cflags libgvc) \
                -I${pkgs.llvmPackages.libclang.lib}/lib/clang/19/include \
                -I${pkgs.glibc.dev}/include"
            '';

          cargoLock = {
            lockFile = ./Cargo.lock;
          };
        };

        goPackage = pkgs.buildGoModule {
          pname = "storyteller-api";
          version = "0.1.0";
          src = ./api;
          vendorHash = "sha256-UvgZE4gGd5rYazQV+2/ZkGlcrIwsdKwnZ4BOV1KSTmM=";
        };

        # Static Graphviz build derivation

        fullVersion = pkgs.llvmPackages.libclang.lib.version;
        majorVersion =
          let matches = builtins.match "^([0-9]+)" fullVersion;
          in if matches == null then fullVersion else matches[0];

        staticGraphviz = pkgs.stdenv.mkDerivation rec {
          pname = "static-graphviz-libs-only";
          version = "13.0.0";

          src = pkgs.fetchFromGitLab {
            owner = "graphviz";
            repo = "graphviz";
            rev = version;
            hash = "sha256-wDjTtI/TyrpXgN4Jk5m0Q9tCNr1lsDQ69nxMi24JWpE=";
          };

          nativeBuildInputs = [
            pkgs.autoreconfHook
            pkgs.pkg-config
            pkgs.makeWrapper
            pkgs.python3
            pkgs.bison
            pkgs.flex
          ];

          buildInputs = [
            pkgs.libpng
            pkgs.libjpeg
            pkgs.expat
            pkgs.fontconfig
            pkgs.gd
            pkgs.gts
            pkgs.pango
          ];

          preAutoreconf = ''
            ./autogen.sh
          '';

          configureFlags = [
            "--with-pic"
            "--without-x"
            "--disable-x"

            "--enable-gvc"
            "--enable-plugin"

            "--enable-ast"
            "--enable-common"
            "--enable-fdpgen"
            "--enable-label"
            "--enable-mingle"
            "--enable-pack"
            "--enable-sfdpgen"
            "--enable-twopigen"
            "--enable-xdot"

            "--enable-cdt"
            "--enable-dotgen"
            "--enable-glcomp"
            "--enable-neatogen"
            "--enable-patchwork"
            "--enable-sfio"
            "--enable-util"
            "--enable-cgraph"
            "--enable-edgepaint"
            "--enable-ortho"
            "--enable-pathplan"
            "--enable-sparse"
            "--enable-vmalloc"
            "--enable-circogen"
            "--enable-expr"
            "--enable-gvpr"
            "--enable-osage"
            "--enable-rbtree"
            "--enable-topfish"
            "--enable-vpsc"
          ];

          doCheck = false;
          doInstallCheck = false;

          postInstall = ''
            mkdir -p $out/lib
            find . -name '*.a' -exec cp -v {} $out/lib \;

            mkdir -p $out/include
            find . -name '*.h' -exec cp -v --parents {} $out/include \;
          '';

          enableParallelBuilding = true;
        };

        nixosModule = { config, pkgs, lib, ...}: {
          systemd.services.storyteller-api = {
            description = "Storyteller API Service";
            wantedBy = [ "multi-user.target" ];
            serviceConfig.ExecStart = "${goPackage}/bin/storyteller-api";
            serviceConfig.Restart = "on-failure";
          };

          systemd.services.storyteller = {
            description = "Storyteller UI Service";
            wantedBy = [ "multi-user.target" ];
            serviceConfig.ExecStart = "${rustPackage}/bin/storyteller-ui";
            serviceConfig.Restart = "on-failure";
          }; 
        };

      in {
        packages = {
          # Combined default package: Rust + Go + Graphviz
          default = pkgs.symlinkJoin {
            name = "storyteller-with-graphviz";
            paths = [ rustPackage goPackage staticGraphviz ];
          };

          # Individual packages available
          storyteller-ui = rustPackage;
          storyteller-api = goPackage;
          static-graphviz = staticGraphviz;
        };

        devShells = {
          default = pkgs.mkShell {
            buildInputs = [
              rustToolchain
              pkgs.go
              pkgs.apacheHttpd
              pkgs.pkg-config
              pkgs.openssl
              pkgs.gorm-gentool
              staticGraphviz
              pkgs.pkg-config
              pkgs.libclang
              pkgs.clang
              pkgs.zlib
              pkgs.libxml2
              pkgs.expat
              pkgs.llvmPackages.libclang
            ];

            shellHook = ''
              echo "Rust and Go dev environment loaded with Static Graphviz support."
              echo "Using clang version ${pkgs.llvmPackages.libclang.lib.version}"
              export LIBCLANG_PATH=${pkgs.llvmPackages.libclang.lib}/lib
              export INCLUDE_DIR=${staticGraphviz}/include/
              export PKG_CONFIG_PATH=${staticGraphviz}/lib/pkgconfig
              export BINDGEN_EXTRA_CLANG_ARGS="$(pkg-config --cflags libgvc) \
                -I${pkgs.llvmPackages.libclang.lib}/lib/clang/19/include \
                -I${pkgs.glibc.dev}/include"
            '';
          };
        };
      }
    );
}