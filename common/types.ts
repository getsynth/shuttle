/*
 Generated by typeshare 1.11.0
*/

/** Helper type for typeshare */
export type SecretStoreT = Record<string, string>;

export interface AddCertificateRequest {
	subject: string;
}

export interface ApiError {
	message: string;
	status_code: number;
}

export interface BuildArgsRustBeta {
	/** Version of shuttle-runtime used by this crate */
	shuttle_runtime_version?: string;
	/** Use the built in cargo chef setup for caching */
	cargo_chef: boolean;
	/** Build with the built in `cargo build` setup */
	cargo_build: boolean;
	/** The cargo package name to compile */
	package_name?: string;
	/** The cargo binary name to compile */
	binary_name?: string;
	/** comma-separated list of features to activate */
	features?: string;
	/** Passed on to `cargo build` */
	no_default_features: boolean;
	/** Use the mold linker */
	mold: boolean;
}

export interface BuildMetaBeta {
	git_commit_id?: string;
	git_commit_msg?: string;
	git_branch?: string;
	git_dirty?: boolean;
}

export interface CertificateResponse {
	id: string;
	subject: string;
	serial_hex: string;
	not_after: string;
}

/** Holds the data for building a database connection string on the Beta platform. */
export interface DatabaseInfoBeta {
	engine: string;
	role_name: string;
	role_password: string;
	database_name: string;
	port: string;
	hostname: string;
	/**
	 * The RDS instance name, which is required for deleting provisioned RDS instances, it's
	 * optional because it isn't needed for shared PG deletion.
	 */
	instance_name?: string;
}

export interface DeleteCertificateRequest {
	subject: string;
}

export enum DeploymentStateBeta {
	Pending = "pending",
	Building = "building",
	Running = "running",
	InProgress = "inprogress",
	Stopped = "stopped",
	Stopping = "stopping",
	Failed = "failed",
	/** Fallback */
	Unknown = "unknown",
}

export interface DeploymentResponseBeta {
	id: string;
	state: DeploymentStateBeta;
	created_at: string;
	updated_at: string;
	/** URIs where this deployment can currently be reached (only relevant for Running state) */
	uris: string[];
}

export interface DeploymentListResponseBeta {
	deployments: DeploymentResponseBeta[];
}

export type BuildArgsBeta = 
	| { type: "Rust", content: BuildArgsRustBeta }
	| { type: "Unknown", content?: undefined };

export interface DeploymentRequestBuildArchiveBeta {
	/** The S3 object version ID of the archive to use */
	archive_version_id: string;
	build_args?: BuildArgsBeta;
	/**
	 * Secrets to add before this deployment.
	 * TODO: Remove this in favour of a separate secrets uploading action.
	 */
	secrets?: Record<string, string>;
	build_meta?: BuildMetaBeta;
}

export interface DeploymentRequestImageBeta {
	image: string;
	/** TODO: Remove this in favour of a separate secrets uploading action. */
	secrets?: Record<string, string>;
}

export interface LogItemBeta {
	timestamp: string;
	/** Which container / log stream this line came from */
	source: string;
	line: string;
}

export interface LogsResponseBeta {
	logs: LogItemBeta[];
}

export interface ProjectResponseBeta {
	id: string;
	/** Project owner */
	user_id: string;
	name: string;
	created_at: string;
	/** State of the current deployment if one exists (something has been deployed). */
	deployment_state?: DeploymentStateBeta;
	/** URIs where running deployments can be reached */
	uris: string[];
}

export interface ProjectListResponseBeta {
	projects: ProjectResponseBeta[];
}

export enum ResourceTypeBeta {
	DatabaseSharedPostgres = "database::shared::postgres",
	DatabaseAwsRdsPostgres = "database::aws_rds::postgres",
	DatabaseAwsRdsMysql = "database::aws_rds::mysql",
	DatabaseAwsRdsMariaDB = "database::aws_rds::mariadb",
	/** (Will probably be removed) */
	Secrets = "secrets",
	/** Local provisioner only */
	Container = "container",
}

export interface ProvisionResourceRequestBeta {
	/** The type of this resource */
	type: ResourceTypeBeta;
	/**
	 * The config used when creating this resource.
	 * Use `Self::r#type` to know how to parse this data.
	 */
	config: any;
}

/** The resource state represents the stage of the provisioning process the resource is in. */
export enum ResourceState {
	Authorizing = "authorizing",
	Provisioning = "provisioning",
	Failed = "failed",
	Ready = "ready",
	Deleting = "deleting",
	Deleted = "deleted",
}

export interface ResourceResponseBeta {
	type: ResourceTypeBeta;
	state: ResourceState;
	/** The config used when creating this resource. Use the `r#type` to know how to parse this data. */
	config: any;
	/** The output type for this resource, if state is Ready. Use the `r#type` to know how to parse this data. */
	output: any;
}

export interface ResourceListResponseBeta {
	resources: ResourceResponseBeta[];
}

export enum SubscriptionType {
	Pro = "pro",
	Rds = "rds",
}

export interface Subscription {
	id: string;
	type: SubscriptionType;
	quantity: number;
	created_at: string;
	updated_at: string;
}

export interface SubscriptionRequest {
	id: string;
	type: SubscriptionType;
	quantity: number;
}

export interface UploadArchiveResponseBeta {
	/** The S3 object version ID of the uploaded object */
	archive_version_id: string;
}

export enum AccountTier {
	Basic = "basic",
	PendingPaymentPro = "pendingpaymentpro",
	CancelledPro = "cancelledpro",
	Pro = "pro",
	Team = "team",
	Admin = "admin",
	Deployer = "deployer",
}

export interface UserResponse {
	name: string;
	id: string;
	key: string;
	account_tier: AccountTier;
	subscriptions: Subscription[];
	has_access_to_beta: boolean;
}

export type DeploymentRequestBeta = 
	/** Build an image from the source code in an attached zip archive */
	| { type: "BuildArchive", content: DeploymentRequestBuildArchiveBeta }
	/** Use this image directly. Can be used to skip the build step. */
	| { type: "Image", content: DeploymentRequestImageBeta };

