{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    ipso.url = "github:LightAndLight/ipso";
  };
  outputs = { self, nixpkgs, flake-utils, ipso }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
      
      in {
        devShell =
          pkgs.mkShell {
            buildInputs = [
              ipso.defaultPackage.${system}
            ];
          };
      }
    );
}
