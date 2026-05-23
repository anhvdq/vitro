import { clsx, type ClassValue } from 'clsx';
import { twMerge } from 'tailwind-merge';

export function cn(...inputs: ClassValue[]) {
	return twMerge(clsx(inputs));
}

type PaginationArgs = {
	page?: number;
	limit?: number;
};

export function paginate(data: any[], { page = 1, limit = 10 }: PaginationArgs) {
	const safePage = Number(page);
	const safeLimit = Number(limit);

	if (!Number.isInteger(safePage) || safePage <= 0) {
		return [];
	}

	if (!Number.isInteger(safeLimit) || safeLimit <= 0) {
		return data;
	}

	const start = (safePage - 1) * safeLimit;
	const end = safePage * safeLimit;

	if (start >= data.length) {
		return [];
	}

	return data.slice(start, end);
}
