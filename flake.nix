{
  outputs =
    { nixpkgs, ... }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
        config.allowUnfree = true;
      };
      inherit (pkgs) stdenv;

      toSystem =
        flakeObj:
        let
          createAttr = (parts: attr: parts // { ${attr}.${system} = flakeObj.${attr}; });
        in
        builtins.foldl' createAttr { } (builtins.attrNames flakeObj);

      NIX_LD_LIBRARY_PATH = [ stdenv.cc.cc ];
    in
    toSystem {

      devShells.default = pkgs.mkShell {
        name = "save";

        buildInputs = with pkgs; [
          rustup
          openssl
          pkg-config
          nodejs
        ];

        shellHook = ''
          export LD_LIBRARY_PATH=NIX_LD_LIBRARY_PATH
          export PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig";
          export SHELL="${pkgs.zsh}/bin/zsh";

          export PATH="$HOME/.cargo/bin:$PATH"
          "$SHELL"
        '';
      };
    };
}
