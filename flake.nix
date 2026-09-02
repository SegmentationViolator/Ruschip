{
    inputs = {
        crane.url = "github:ipetkov/crane";
        nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
        systems.url = "github:nix-systems/default";
    };

    outputs =
        {
            self,
            nixpkgs,
            crane,
            systems,
            ...
        }:
            let
                aliases = eachSystem (system: rec {
                    pkgs = import nixpkgs {
                        inherit system;
                    };

                    craneLib = crane.mkLib pkgs;

                    src = craneLib.cleanCargoSource ./.;

                    cargoArguments = {
                        inherit src;
                        strictDeps = true;

                        nativeBuildInputs = with pkgs; [
                            pkg-config
                        ];

                        buildInputs = with pkgs; lib.optionals stdenv.buildPlatform.isLinux [
                            alsa-lib
                            egl-wayland
                            libGL
                            libxkbcommon
                            wayland
                        ] ++ lib.optionals stdenv.buildPlatform.isDarwin [
                            libiconv
                        ];

                        env = {
                            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
                                pkgs.alsa-lib
                                pkgs.libGL
                                pkgs.libxkbcommon
                                pkgs.wayland
                            ];
                        };
                    };

                    cargoArtifacts = craneLib.buildDepsOnly cargoArguments;

                });

                eachSystem = nixpkgs.lib.genAttrs (import systems);
            in
            {
                checks = eachSystem (system: {
                    crate-clippy = aliases.${system}.craneLib.cargoClippy (
                        aliases.${system}.cargoArguments
                        // {
                            cargoArtifacts = aliases.${system}.cargoArtifacts;
                            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
                        }
                    );
                });

                packages = eachSystem (system: {
                    default = aliases.${system}.craneLib.buildPackage (
                        aliases.${system}.cargoArguments
                        // {
                            cargoArtifacts = aliases.${system}.cargoArtifacts;
                        }

                    );
                });

                devShell = eachSystem (system: aliases.${system}.craneLib.devShell {
                    checks = self.checks.${system};

                    nativeBuildInputs = with aliases.${system}.pkgs; [
                        pkg-config
                    ];

                    buildInputs = with aliases.${system}.pkgs; lib.optionals stdenv.buildPlatform.isLinux [
                        alsa-lib
                        egl-wayland
                        libGL
                        libxkbcommon
                        wayland
                    ] ++ lib.optionals stdenv.buildPlatform.isDarwin [
                        libiconv
                    ];

                    env = {
                        LD_LIBRARY_PATH = aliases.${system}.pkgs.lib.makeLibraryPath [
                            aliases.${system}.pkgs.alsa-lib
                            aliases.${system}.pkgs.libGL
                            aliases.${system}.pkgs.libxkbcommon
                            aliases.${system}.pkgs.wayland
                        ];
                    };
                });
            };
}
