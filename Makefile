PREFIX ?= /usr
CARGO_BUILD_ARGS ?= --release
TARGET_DIR ?= release

build:
	@cargo build $(CARGO_BUILD_ARGS)
	@glib-compile-schemas ./schemas
	@./scripts/translations.sh update

install: build
	@mkdir -p "$(PREFIX)/share/icons/hicolor/scalable/apps/"
	@mkdir -p "$(PREFIX)/share/applications/"
	@mkdir -p "$(PREFIX)/share/glib-2.0/schemas/"
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
	@glib-compile-schemas "$(PREFIX)/share/glib-2.0/schemas/"
	@desktop-file-install -m 0644 --dir="$(PREFIX)/share/applications/" data/de.fin_ger.OvGUCanteen.desktop
	@install -s -m 0755 "target/$(TARGET_DIR)/gnome-ovgu-canteen" "$(PREFIX)/bin/"
	@./scripts/translations.sh install
	@update-desktop-database "$(PREFIX)/share/applications"
	@gtk-update-icon-cache

run:
	@$(MAKE) -s install PREFIX=$(HOME)/.local CARGO_BUILD_ARGS= TARGET_DIR=debug
	@gnome-ovgu-canteen

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
	@rm "$(PREFIX)/bin/gnome-ovgu-canteen"
	@./scripts/translations.sh uninstall

clean:
	@$(MAKE) -s uninstall PREFIX=$(HOME)/.local
	@echo "Installation files have been cleaned. To also clean cargo build files run 'cargo clean'"
