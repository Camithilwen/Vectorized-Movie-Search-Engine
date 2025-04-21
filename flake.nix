{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = {
    self,
    nixpkgs,
    ...
  } @ inputs: {
    devShells = (
      nixpkgs.lib.genAttrs ["x86_64-linux"]
      (system: let
        pkgs = nixpkgs.legacyPackages.${system};
      in {
        default = pkgs.mkShell {
          packages = with pkgs; [
            rustup
            rustc
            pkg-config
            fontconfig
            openssl
          ];
        };

      })
    );
  };
}
