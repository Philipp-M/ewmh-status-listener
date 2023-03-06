{
  description = "Simple EWMH status listener that continuously gives json output of the current desktop state of an EWMH compatible window manager";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.flake-utils.follows = "flake-utils";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, flake-utils, nixpkgs, rust-overlay }:
    let
      inherit (builtins) substring;
      inherit (nixpkgs) lib;

      mtime = self.lastModifiedDate;
      date = "${substring 0 4 mtime}-${substring 4 2 mtime}-${substring 6 2 mtime}";
      rev = self.rev or (throw "Git changes are not committed");

      mkEwmhStatusListener = { rustPlatform, xorg, ... }:
        rustPlatform.buildRustPackage {
          pname = "ewmh-status-listener";
          version = "unstable-${date}";
          src = self;
          cargoLock.lockFile = self + "/Cargo.lock";

          buildInputs = [ xorg.libxcb ];

          CFG_RELEASE = "git-${rev}";
        };
    in
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          rustPkgs = rust-overlay.packages.${system};
        in
        {
          packages = rec {
            default = ewmh-status-listener;
            ewmh-status-listener = pkgs.callPackage mkEwmhStatusListener { };
          };

          devShells.default = pkgs.mkShell {
            nativeBuildInputs = with pkgs; let
              vers = lib.splitVersion rustc.version;
              rustVersion = "${lib.elemAt vers 0}_${lib.elemAt vers 1}_${lib.elemAt vers 2}";
            in
            [
              # Follows nixpkgs's version of rustc.
              rustPkgs."rust_${rustVersion}"
              nixpkgs-fmt
              xorg.libxcb
            ];

            RUST_BACKTRACE = "short";
            NIXPKGS = nixpkgs;
          };
        })
    // {
      overlays = rec {
        default = ewmh-status-listener;
        ewmh-status-listener = final: prev: {
          ewmh-status-listener = final.callPackage mkEwmhStatusListener { };
        };
      };
    };
}

