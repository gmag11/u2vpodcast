export const isValidEmail = (email: string) => {
	const EMAIL_REGEX =
		/[a-z0-9!#$%&'*+/=?^_`{|}~-]+(?:\.[a-z0-9!#$%&'*+/=?^_`{|}~-]+)*@(?:[a-z0-9](?:[a-z0-9-]*[a-z0-9])?\.)+[a-z0-9](?:[a-z0-9-]*[a-z0-9])?/;
	return EMAIL_REGEX.test(email.trim());
};

export const isValidPasswordStrong = (password: string) => {
	const strongRegex = new RegExp('^(?=.*[a-z])(?=.*[A-Z])(?=.*[0-9])(?=.*[!@#$%^&*])(?=.{8,})');
	return strongRegex.test(password.trim());
};

export const isValidPasswordMedium = (password: string) => {
	const mediumRegex = new RegExp(
		'^(((?=.*[a-z])(?=.*[A-Z]))|((?=.*[a-z])(?=.*[0-9]))|((?=.*[A-Z])(?=.*[0-9])))(?=.{6,})'
	);
	return mediumRegex.test(password.trim());
};
