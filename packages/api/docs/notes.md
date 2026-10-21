All items below were resolved on this branch.

- [x] Update format to snake_case for dataset | asset name
- [x] Create asset on dataset details view -> make dataset ID immutable
- [x] Dataset list view -> display asset's description (cutoff if too long) instead of Dataset ID column
- [x] Dataset list view breadcrumb -> currently # > Datasets > Datasets which is duplicated, should be # > Datasets
- [x] Dataset details breadcrumb -> Display dataset name instead of "Dataset"
- [x] When viewing an asset from dataset details view, breadcrumb should be # > Datasets > dataset name > asset name. Currently it's Assets > Asset
- [x] When view details of an asset from the Asset list view -> breadcrumb should be Assets > asset name
- [x] If dataset is archived -> mute the NEW button to prevent creating new asset
- [x] Asset list view -> make dataset_id optional, by default, it should fetch all assets from all datasets. Put the dataset selection as a filter similar to status filter
- [x] Asset upload file then save the file metadata into assets table
