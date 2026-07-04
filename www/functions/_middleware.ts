import { buildLegacyRedirectMap } from '../legacy-redirects.mjs';

const legacyRedirectTargets = buildLegacyRedirectMap();

export const onRequest: PagesFunction = async (context) => {
	const pathname = new URL(context.request.url).pathname;
	const destination = legacyRedirectTargets[pathname];

	if (destination) {
		return Response.redirect(new URL(destination, context.request.url), 301);
	}

	return context.next();
};
