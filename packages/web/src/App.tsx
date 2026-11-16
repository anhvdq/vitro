import { Route, Router } from '@solidjs/router';
import { QueryClient, QueryClientProvider } from '@tanstack/solid-query';
import SignInPage from './features/auth/SignInPage';
import HomePage from './features/home/HomePage';

export default function App() {
	const queryClient = new QueryClient();
	return (
		<QueryClientProvider client={queryClient}>
			<Router>
				<Route path="/" component={HomePage} />
				<Route path="/sign-in" component={SignInPage} />
			</Router>
		</QueryClientProvider>
	);
}
