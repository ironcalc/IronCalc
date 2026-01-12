import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import translationDE from "./locale/de_de.json";
import translationEN from "./locale/en_us.json";
import translationES from "./locale/es_es.json";
import translationFR from "./locale/fr_fr.json";
import translationIT from "./locale/it_it.json";

const resources = {
  "en-US": { translation: translationEN },
  "es-ES": { translation: translationES },
  "fr-FR": { translation: translationFR },
  "de-DE": { translation: translationDE },
  "it-IT": { translation: translationIT },
};

i18n.use(initReactI18next).init({
  resources,
  lng: "en-US",
  interpolation: {
    escapeValue: false,
  },
});

export default i18n;
