import { apiClient } from './client'

export type ResourceType = 'content' | 'schedule' | 'node'
export type PermissionLevel = 'read' | 'write' | 'admin'

export interface Permission {
  id: number
  user_id: number
  resource_type: ResourceType
  resource_id: number // 0 = wildcard ("All <type>")
  permission_level: PermissionLevel
  created_at: string
}

export interface NewPermission {
  user_id: number
  resource_type: ResourceType
  resource_id: number
  permission_level: PermissionLevel
}

export const getPermissions = () =>
  apiClient.get<Permission[]>('/api/permissions').then(res => res.data)

export const createPermission = (data: NewPermission) =>
  apiClient.post<Permission>('/api/permissions', data).then(res => res.data)

export const deletePermission = (id: number) =>
  apiClient.delete(`/api/permissions/${id}`)
