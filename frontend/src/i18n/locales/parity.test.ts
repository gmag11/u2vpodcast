import { describe, expect, it } from 'vitest';
import en from './en.json';
import es from './es.json';

type JsonRecord = { [key: string]: string | JsonRecord };

function flattenKeys(obj: JsonRecord, prefix = ''): string[] {
	const keys: string[] = [];
	for (const [key, value] of Object.entries(obj)) {
		const path = prefix ? `${prefix}.${key}` : key;
		if (value !== null && typeof value === 'object') {
			keys.push(...flattenKeys(value as JsonRecord, path));
		} else {
			keys.push(path);
		}
	}
	return keys;
}

describe('i18n catalogues', () => {
	it('es covers every key present in en', () => {
		const enKeys = flattenKeys(en as JsonRecord).sort();
		const esKeys = flattenKeys(es as JsonRecord).sort();
		const missing = enKeys.filter((key) => !esKeys.includes(key));
		expect(missing).toEqual([]);
	});

	it('does not contain extra keys in es that are absent from en', () => {
		const enKeys = flattenKeys(en as JsonRecord).sort();
		const esKeys = flattenKeys(es as JsonRecord).sort();
		const extra = esKeys.filter((key) => !enKeys.includes(key));
		expect(extra).toEqual([]);
	});

	it('keeps interpolation placeholders aligned between locales', () => {
		expect(es.card.syncTooltip).toContain('{age}');
		expect(es.card.syncTooltip).toContain('{status}');
	});
});
