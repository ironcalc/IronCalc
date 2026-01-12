import { useTranslation } from "react-i18next";

function LanguageSelector() {
  const { i18n } = useTranslation();

  return (
    <select
      value={i18n.language}
      onChange={(e) => i18n.changeLanguage(e.target.value)}
    >
      <option value="en-US">English</option>
      <option value="es-ES">Español</option>
      <option value="fr-FR">Français</option>
      <option value="de-DE">Deutsch</option>
      <option value="it-IT">Italiano</option>
    </select>
  );
}

export default LanguageSelector;
