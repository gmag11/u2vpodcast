// Shared i18n instance for component tests. Re-uses the app singleton so the
// default locale (`en`) is pinned and assertions never depend on the machine's
// `navigator.language`.
export { default as testI18n } from '@/i18n';
