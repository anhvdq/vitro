import * as React from 'react';
import { ChartColumnIncreasing, CircleGauge, Database, LayoutList, Settings } from 'lucide-react';

import { NavMain } from '@/components/nav-main';
import { NavUser } from '@/components/nav-user';
import {
	Sidebar,
	SidebarContent,
	SidebarFooter,
	SidebarHeader,
	SidebarMenu,
	SidebarMenuButton,
	SidebarMenuItem
} from '@/components/ui/sidebar';
import { useAuthStore } from '@/lib/state/auth';

const navMain = [
	{
		title: 'Dashboard',
		url: '/dashboard',
		icon: ChartColumnIncreasing
	},
	{
		title: 'Catalog',
		url: '',
		icon: Database,
		isActive: true,
		items: [
			{
				title: 'Datasets',
				url: '/datasets'
			},
			{
				title: 'Assets',
				url: '/assets'
			}
		]
	},
	{
		title: 'Tasks',
		url: '/tasks',
		icon: LayoutList
	},
	{
		title: 'Settings',
		url: '/settings',
		icon: Settings
	}
];

export function LeftSidebar({ ...props }: React.ComponentProps<typeof Sidebar>) {
	const auth = useAuthStore((s) => s.auth);
	if (!auth) return null;

	return (
		<Sidebar variant="inset" {...props}>
			<SidebarHeader>
				<SidebarMenu>
					<SidebarMenuItem>
						<SidebarMenuButton size="lg" asChild>
							<a href="#">
								<div className="bg-sidebar-primary text-sidebar-primary-foreground flex aspect-square size-8 items-center justify-center rounded-lg">
									<CircleGauge className="size-5" />
								</div>
								<div className="grid flex-1 text-left text-sm leading-tight">
									<span className="truncate font-semibold">Vitro</span>
									<span className="truncate text-xs">Data Profiling Platform</span>
								</div>
							</a>
						</SidebarMenuButton>
					</SidebarMenuItem>
				</SidebarMenu>
			</SidebarHeader>
			<SidebarContent>
				<NavMain items={navMain} />
			</SidebarContent>
			<SidebarFooter>
				<NavUser user={{ name: auth.user.name, email: auth.user.email, avatar: '/avatar.jpg' }} />
			</SidebarFooter>
		</Sidebar>
	);
}
