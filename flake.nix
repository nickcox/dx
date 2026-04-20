{
  description = "cdex packaging for dx CLI";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
    in
    {
      packages = forAllSystems (system:
        let
          pkgs = import nixpkgs { inherit system; };
        in
        rec {
          cdex = pkgs.rustPlatform.buildRustPackage {
            pname = "cdex";
            version = "0.1.0";
            src = ./.;
            cargoLock = {
              lockFile = ./Cargo.lock;
            };
            doCheck = false;
          };
          default = cdex;
        });

      apps = forAllSystems (system:
        let
          pkg = self.packages.${system}.cdex;
        in
        {
          dx = {
            type = "app";
            program = "${pkg}/bin/dx";
          };
          default = self.apps.${system}.dx;
        });

      devShells = forAllSystems (system:
        let
          pkgs = import nixpkgs { inherit system; };
        in
        {
          default = pkgs.mkShell {
            packages = with pkgs; [
              rustc
              cargo
            ];
          };
        });
    };
}
