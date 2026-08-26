import { createFileRoute, useRouter } from '@tanstack/react-router';
import { useEffect } from 'react';
import { useAuthStore } from '@/lib/state/auth';

export const Route = createFileRoute('/')({
	component: RouteComponent
});

function RouteComponent() {
	const router = useRouter();
	const hydrated = useAuthStore((s) => s.hydrated);
	const auth = useAuthStore((s) => s.auth);

	useEffect(() => {
		if (!hydrated) return;
		router.navigate({ to: auth ? '/dashboard' : '/login' });
	}, [hydrated, auth, router]);

	return null;
}
