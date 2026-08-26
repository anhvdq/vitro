const examplePath = `${process.cwd()}/.env.example`;
const envPath = `${process.cwd()}/.env`;

if (!(await Bun.file(examplePath).exists())) {
	throw new Error(`Missing environment template: ${examplePath}`);
}

if (await Bun.file(envPath).exists()) {
	console.log(`Skipped existing ${envPath}`);
} else {
	await Bun.write(envPath, Bun.file(examplePath));
	console.log(`Created ${envPath}`);
}
