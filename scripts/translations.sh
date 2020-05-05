#!/bin/bash

set -e

MSGFMT="${MSGFMT:-/usr/bin/msgfmt}"
PREFIX="${PREFIX:-/usr}"

case $1 in
    update)
        echo "Updating translations..."

        (
            cd po

            for po in ./*.po
            do
                lang="${po%.po}"
                mkdir -p "${lang}/LC_MESSAGES"
                "${MSGFMT}" --output "${lang}/LC_MESSAGES/ovgu-canteen-gtk.mo" "$po"
            done
        )
        ;;
    install)
        echo "Installing translations..."

        (
            cd po

            for po in ./*.po
            do
                lang="${po%.po}"
                mkdir -p "${PREFIX}/share/locale/${lang}/LC_MESSAGES"
                install -m 0644 "${lang}/LC_MESSAGES/ovgu-canteen-gtk.mo" "${PREFIX}/share/locale/${lang}/LC_MESSAGES"
            done
        )
        ;;
    uninstall)
        echo "Uninstalling translations..."

        (
            cd po

            for po in ./*.po
            do
                lang="${po%.po}"
                rm "${PREFIX}/share/locale/${lang}/LC_MESSAGES/ovgu-canteen-gtk.mo"
            done
        )
        ;;
    *)
        echo "Unknown action '$1'!" 1>&2
        exit 1
esac
