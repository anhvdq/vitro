import { create } from 'zustand';

type BreadcrumbStore = {
	labels: Record<string, string>;
	setLabel: (routeId: string, label: string) => void;
};

export const useBreadcrumbStore = create<BreadcrumbStore>((set) => ({
	labels: {},
	setLabel: (routeId, label) => set((state) => ({ labels: { ...state.labels, [routeId]: label } }))
}));
