import { create } from 'zustand';

type UiStore = {
	selectedDatasetId: string | null;
	setSelectedDatasetId: (id: string) => void;
};

export const useUiStore = create<UiStore>((set) => ({
	selectedDatasetId: null,
	setSelectedDatasetId: (id) => set({ selectedDatasetId: id })
}));
