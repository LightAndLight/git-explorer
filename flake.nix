{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nix-filter.url = "github:numtide/nix-filter";
    ipso.url = "github:LightAndLight/ipso?commit=e7bf506bd8f85f00c7b00b795b587f79b5bb5d9d";
  };
  outputs = { self, nixpkgs, flake-utils, nix-filter, ipso }:
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
        
        defaultPackage = pkgs.stdenv.mkDerivation {
          name = "git-explorer";
          src = nix-filter.lib {
            root = ./.;
            include = [ "gitex" ];
          };
          buildInputs = [ ipso.defaultPackage.${system} ];
          buildPhase = "true";
          installPhase = ''
            mkdir -p $out/bin
            cp gitex $out/bin
            chmod +x $out/bin/gitex
          '';
        };
      }
    );
}
