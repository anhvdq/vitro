import z from 'zod';

export const UserSchema = z.object({
	id: z.string(),
	name: z.string(),
	age: z.number().nullable(),
	email: z.string(),
	default_workspace_id: z.string()
});

export type User = z.infer<typeof UserSchema>;

const AuthSessionDataSchema = z.object({
	access_token: z.string(),
	user: UserSchema
});

export const AuthSessionResponseSchema = z.object({
	data: AuthSessionDataSchema
});

export type AuthSessionResponse = z.infer<typeof AuthSessionResponseSchema>;

export const AuthMeResponseSchema = z.object({
	data: UserSchema
});

export type AuthMeResponse = z.infer<typeof AuthMeResponseSchema>;

export type AuthState = {
	accessToken: string;
	workspaceId: string;
	user: User;
};
