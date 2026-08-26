import { cn } from '@/lib/utils';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Link, useRouter } from '@tanstack/react-router';
import { useState } from 'react';
import { exchangeGoogleToken, signupWithPassword } from '@/lib/auth/api';
import { VitroApiError } from '@/lib/api/errors';
import { useAuthStore } from '@/lib/state/auth';
import { toast } from 'sonner';

export function SignupForm({ className, ...props }: React.ComponentPropsWithoutRef<'form'>) {
	const router = useRouter();
	const setAuth = useAuthStore((s) => s.setAuth);
	const [name, setName] = useState('');
	const [email, setEmail] = useState('');
	const [password, setPassword] = useState('');

	return (
		<form
			className={cn('flex flex-col gap-6', className)}
			onSubmit={(e) => {
				e.preventDefault();
				(async () => {
					const auth = await signupWithPassword({
						name: name.trim(),
						email: email.trim(),
						password
					});
					setAuth(auth);
					router.navigate({ to: '/dashboard' });
				})().catch(() => {
					toast.error('Sign up failed');
				});
			}}
			{...props}
		>
			<div className="flex flex-col items-center gap-2 text-center">
				<h1 className="text-2xl font-bold">Create your account</h1>
				<p className="text-muted-foreground text-sm text-balance">
					Enter your details below to sign up
				</p>
			</div>
			<div className="grid gap-6">
				<div className="grid gap-2">
					<Label htmlFor="name">Name</Label>
					<Input id="name" value={name} onChange={(e) => setName(e.target.value)} required />
				</div>
				<div className="grid gap-2">
					<Label htmlFor="email">Email</Label>
					<Input
						id="email"
						type="email"
						placeholder="m@example.com"
						value={email}
						onChange={(e) => setEmail(e.target.value)}
						required
					/>
				</div>
				<div className="grid gap-2">
					<Label htmlFor="password">Password</Label>
					<Input
						id="password"
						type="password"
						value={password}
						onChange={(e) => setPassword(e.target.value)}
						required
					/>
				</div>
				<Button type="submit" className="w-full">
					Sign up
				</Button>
				<div className="after:border-border relative text-center text-sm after:absolute after:inset-0 after:top-1/2 after:z-0 after:flex after:items-center after:border-t">
					<span className="bg-background text-muted-foreground relative z-10 px-2">
						Or continue with
					</span>
				</div>
				<Button
					variant="outline"
					className="w-full"
					type="button"
					onClick={() => {
						exchangeGoogleToken('placeholder_google_token')
							.then((auth) => {
								setAuth(auth);
								router.navigate({ to: '/dashboard' });
							})
							.catch((err: unknown) => {
								if (err instanceof VitroApiError && err.status === 501) {
									toast.error('Google login is not implemented yet.');
									return;
								}
								toast.error('Google login failed');
							});
					}}
				>
					Login with Google
				</Button>
			</div>
			<div className="text-center text-sm">
				Already have an account?{' '}
				<Link to="/login" className="underline underline-offset-4">
					Login
				</Link>
			</div>
		</form>
	);
}
