{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-22.11";
    systems.url = "github:nix-systems/default";
    devenv.url = "github:ELD/devenv/fix-rust-with-processes";

    # Rust support
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, devenv, systems, ... } @ inputs:
    let
      forEachSystem = nixpkgs.lib.genAttrs (import systems);
    in
    {
      devShells = forEachSystem
        (system:
          let
            pkgs = nixpkgs.legacyPackages.${system};
          in
          {
            default = devenv.lib.mkShell {
              inherit inputs pkgs;
              modules = [
                {
                  env = {
                    DATABASE_URL = "postgres://localhost:5432/integration";
                  };

                  services = {
                    postgres = {
                      enable = true;
                      listen_addresses = "127.0.0.1";
                    };
                  };

                  languages = {
                    rust.enable = true;
                    rust.version = "stable";
                  };

                  packages = with pkgs; [
                    sccache
                    darwin.DarwinTools
                  ] ++ lib.optionals pkgs.stdenv.isDarwin (with pkgs.darwin.apple_sdk; [
                    frameworks.CoreFoundation
                    frameworks.Security
                    frameworks.SystemConfiguration
                  ]);

                  pre-commit = {
                    settings = {
                      clippy.denyWarnings = true;
                    };

                    hooks = {
                      cargo-check.enable = true;
                      clippy.enable = true;
                      rustfmt.enable = true;
                    };
                  };
                }
              ];
            };
          });
    };
}
