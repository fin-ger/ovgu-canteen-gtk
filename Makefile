PREFIX ?= /usr
CARGO_BUILD_ARGS ?= --release
TARGET_DIR ?= release

build:
	@glib-compile-schemas ./schemas
	@./scripts/translations.sh update
	@cargo build $(CARGO_BUILD_ARGS)
	@./scripts/flatpak-cargo-generator.py Cargo.lock -o flatpak.lock || true

install: build
	@mkdir -p "$(PREFIX)/share/icons/hicolor/scalable/apps/"
	@mkdir -p "$(PREFIX)/share/applications/"
	@mkdir -p "$(PREFIX)/share/glib-2.0/schemas/"
	@mkdir -p "$(PREFIX)/share/metainfo/"
	@mkdir -p "$(PREFIX)/bin"
	@install -m 0644 icons/de.fin_ger.OvGUCanteen.svg "$(PREFIX)/share/icons/hicolor/scalable/apps/"
	@install -m 0644 icons/de.fin_ger.OvGUCanteen.About.svg "$(PREFIX)/share/icons/hicolor/scalable/apps/"
	@install -m 0644 icons/de.fin_ger.OvGUCanteen.Closed.svg "$(PREFIX)/share/icons/hicolor/scalable/apps/"
	@install -m 0644 icons/de.fin_ger.OvGUCanteen.Alcohol.svg "$(PREFIX)/share/icons/hicolor/scalable/apps/"
	@install -m 0644 icons/de.fin_ger.OvGUCanteen.AnimalWelfare.svg "$(PREFIX)/share/icons/hicolor/scalable/apps/"
	@install -m 0644 icons/de.fin_ger.OvGUCanteen.Cattle.svg "$(PREFIX)/share/icons/hicolor/scalable/apps/"
	@install -m 0644 icons/de.fin_ger.OvGUCanteen.Fish.svg "$(PREFIX)/share/icons/hicolor/scalable/apps/"
	@install -m 0644 icons/de.fin_ger.OvGUCanteen.Game.svg "$(PREFIX)/share/icons/hicolor/scalable/apps/"
	@install -m 0644 icons/de.fin_ger.OvGUCanteen.Garlic.svg "$(PREFIX)/share/icons/hicolor/scalable/apps/"
	@install -m 0644 icons/de.fin_ger.OvGUCanteen.Lamb.svg "$(PREFIX)/share/icons/hicolor/scalable/apps/"
	@install -m 0644 icons/de.fin_ger.OvGUCanteen.MensaVital.svg "$(PREFIX)/share/icons/hicolor/scalable/apps/"
	@install -m 0644 icons/de.fin_ger.OvGUCanteen.Organic.svg "$(PREFIX)/share/icons/hicolor/scalable/apps/"
	@install -m 0644 icons/de.fin_ger.OvGUCanteen.Pig.svg "$(PREFIX)/share/icons/hicolor/scalable/apps/"
	@install -m 0644 icons/de.fin_ger.OvGUCanteen.Poultry.svg "$(PREFIX)/share/icons/hicolor/scalable/apps/"
	@install -m 0644 icons/de.fin_ger.OvGUCanteen.SoupOfTheDay.svg "$(PREFIX)/share/icons/hicolor/scalable/apps/"
	@install -m 0644 icons/de.fin_ger.OvGUCanteen.Vegan.svg "$(PREFIX)/share/icons/hicolor/scalable/apps/"
	@install -m 0644 icons/de.fin_ger.OvGUCanteen.Vegetarian.svg "$(PREFIX)/share/icons/hicolor/scalable/apps/"
	@install -m 0644 schemas/de.fin_ger.OvGUCanteen.gschema.xml "$(PREFIX)/share/glib-2.0/schemas/"
	@install -m 0644 data/de.fin_ger.OvGUCanteen.metainfo.xml "$(PREFIX)/share/metainfo/"
	@glib-compile-schemas "$(PREFIX)/share/glib-2.0/schemas/"
	@desktop-file-install -m 0644 --dir="$(PREFIX)/share/applications/" data/de.fin_ger.OvGUCanteen.desktop
	@install -s -m 0755 "target/$(TARGET_DIR)/ovgu-canteen-gtk" "$(PREFIX)/bin/"
	@./scripts/translations.sh install
	@update-desktop-database "$(PREFIX)/share/applications"
	@gtk-update-icon-cache

run:
	@$(MAKE) -s install PREFIX=$(HOME)/.local CARGO_BUILD_ARGS= TARGET_DIR=debug
	@ovgu-canteen-gtk

uninstall:
	@rm "$(PREFIX)/share/icons/hicolor/scalable/apps/de.fin_ger.OvGUCanteen.svg"
	@rm "$(PREFIX)/share/icons/hicolor/scalable/apps/de.fin_ger.OvGUCanteen.About.svg"
	@rm "$(PREFIX)/share/icons/hicolor/scalable/apps/de.fin_ger.OvGUCanteen.Closed.svg"
	@rm "$(PREFIX)/share/icons/hicolor/scalable/apps/de.fin_ger.OvGUCanteen.Alcohol.svg"
	@rm "$(PREFIX)/share/icons/hicolor/scalable/apps/de.fin_ger.OvGUCanteen.AnimalWelfare.svg"
	@rm "$(PREFIX)/share/icons/hicolor/scalable/apps/de.fin_ger.OvGUCanteen.Cattle.svg"
	@rm "$(PREFIX)/share/icons/hicolor/scalable/apps/de.fin_ger.OvGUCanteen.Fish.svg"
	@rm "$(PREFIX)/share/icons/hicolor/scalable/apps/de.fin_ger.OvGUCanteen.Game.svg"
	@rm "$(PREFIX)/share/icons/hicolor/scalable/apps/de.fin_ger.OvGUCanteen.Garlic.svg"
	@rm "$(PREFIX)/share/icons/hicolor/scalable/apps/de.fin_ger.OvGUCanteen.Lamb.svg"
	@rm "$(PREFIX)/share/icons/hicolor/scalable/apps/de.fin_ger.OvGUCanteen.MensaVital.svg"
	@rm "$(PREFIX)/share/icons/hicolor/scalable/apps/de.fin_ger.OvGUCanteen.Organic.svg"
	@rm "$(PREFIX)/share/icons/hicolor/scalable/apps/de.fin_ger.OvGUCanteen.Pig.svg"
	@rm "$(PREFIX)/share/icons/hicolor/scalable/apps/de.fin_ger.OvGUCanteen.Poultry.svg"
	@rm "$(PREFIX)/share/icons/hicolor/scalable/apps/de.fin_ger.OvGUCanteen.SoupOfTheDay.svg"
	@rm "$(PREFIX)/share/icons/hicolor/scalable/apps/de.fin_ger.OvGUCanteen.Vegan.svg"
	@rm "$(PREFIX)/share/icons/hicolor/scalable/apps/de.fin_ger.OvGUCanteen.Vegetarian.svg"
	@rm "$(PREFIX)/share/glib-2.0/schemas/de.fin_ger.OvGUCanteen.gschema.xml"
	@glib-compile-schemas "$(PREFIX)/share/glib-2.0/schemas/"
	@rm "$(PREFIX)/share/applications/de.fin_ger.OvGUCanteen.desktop"
	@rm "$(PREFIX)/bin/ovgu-canteen-gtk"
	@./scripts/translations.sh uninstall

clean:
	@$(MAKE) -s uninstall PREFIX=$(HOME)/.local
	@echo "Installation files have been cleaned. To also clean cargo build files run 'cargo clean'"

flatpak:
	@mkdir -p target/flatpak
	@flatpak-builder --install target/flatpak --force-clean --user -y dist/flatpak/de.fin_ger.OvGUCanteen.json

flatpak-clean:
	@rm -r .flatpak-builder
	@rm -r target/flatpak

release:
	@cargo release
