#! /bin/sh

(
    DIR="$(dirname "$(readlink -f "$0")")"

    cd "$DIR/.." || exit 1

    python3 scripts/flatpak-cargo-generator.py \
            Cargo.lock \
            -o flatpak/generated-sources.json || exit 2

    cd flatpak || exit 1

    flatpak-builder --install repo \
                    de.fin_ger.OvGUCanteen.json \
                    --force-clean --user -y || exit 3
)
