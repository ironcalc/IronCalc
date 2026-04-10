import i18n from "i18next";
import translationDE from "./locale/de_de.json";
import translationEN from "./locale/en_us.json";
import translationES from "./locale/es_es.json";
import translationFR from "./locale/fr_fr.json";
import translationIT from "./locale/it_it.json";

const resources = {
  "en-US": { translation: translationEN },
  "en-GB": { translation: translationEN },
  "es-ES": { translation: translationES },
  "fr-FR": { translation: translationFR },
  "de-DE": { translation: translationDE },
  "it-IT": { translation: translationIT },
};

const instance: typeof i18n = i18n.createInstance({
  resources,
  lng: "en-US",
  interpolation: {
    escapeValue: false,
  },
  showSupportNotice: false,
});

export default instance;
