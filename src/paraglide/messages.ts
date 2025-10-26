import en from '../../messages/en.json' assert { type: 'json' };
import de from '../../messages/de.json' assert { type: 'json' };

type Locale = 'en' | 'de';
type MessageDictionary = typeof en;
type MessageKey = keyof MessageDictionary;

type MessageParams = Record<string, unknown>;

type MessageFunctions = {
        [K in MessageKey]: (params?: MessageParams) => string;
};

const dictionaries: Record<Locale, MessageDictionary> = {
        en,
        de
};

const FALLBACK_LOCALE: Locale = 'en';
let currentLocale: Locale = FALLBACK_LOCALE;

export function setLocale(locale: Locale) {
        if (locale in dictionaries) {
                currentLocale = locale;
                return;
        }

        console.warn(`[i18n] Unsupported locale "${locale}". Falling back to "${FALLBACK_LOCALE}".`);
        currentLocale = FALLBACK_LOCALE;
}

export function getLocale(): Locale {
        return currentLocale;
}

export const availableLocales = Object.keys(dictionaries) as Locale[];

function formatMessage(template: string, params: MessageParams): string {
        return template.replace(/\{(\w+)\}/g, (_, key: string) => {
                if (key in params && params[key] !== undefined && params[key] !== null) {
                        return String(params[key]);
                }

                console.warn(`[i18n] Missing parameter "${key}" for message.`);
                return `{${key}}`;
        });
}

function resolveMessage(locale: Locale, key: MessageKey): string | undefined {
        const dictionary = dictionaries[locale];
        return dictionary[key];
}

const messageProxy: MessageFunctions = new Proxy(
        {},
        {
                get(_target, rawKey) {
                        if (typeof rawKey !== 'string') {
                                return () => String(rawKey);
                        }

                        const key = rawKey as MessageKey;

                        return (params: MessageParams = {}) => {
                                const template =
                                        resolveMessage(currentLocale, key) ??
                                        resolveMessage(FALLBACK_LOCALE, key);

                                if (typeof template !== 'string') {
                                        console.warn(`[i18n] Missing translation for key "${key}".`);
                                        return key;
                                }

                                return formatMessage(template, params);
                        };
                }
        }
) as MessageFunctions;

export type { Locale, MessageFunctions };
export default messageProxy;
