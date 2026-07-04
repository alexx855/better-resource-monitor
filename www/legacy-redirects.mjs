/** @typedef {Record<string, string>} LegacyRedirectMap */

export const comparisonPageKeys = ['vs-stats', 'vs-eul', 'vs-istat-menus'];
export const localizedLocales = ['es', 'pt-br', 'zh-cn'];

/** @returns {LegacyRedirectMap} */
export function buildLegacyRedirectMap() {
	/** @type {LegacyRedirectMap} */
	const redirects = {};

	for (const page of comparisonPageKeys) {
		const canonical = `/comparison/${page}/`;
		redirects[`/${page}`] = canonical;
		redirects[`/${page}/`] = canonical;
		redirects[`/en/${page}`] = canonical;
		redirects[`/en/${page}/`] = canonical;

		for (const locale of localizedLocales) {
			const localized = `/${locale}/comparison/${page}/`;
			redirects[`/${locale}/${page}`] = localized;
			redirects[`/${locale}/${page}/`] = localized;
		}
	}

	return redirects;
}
