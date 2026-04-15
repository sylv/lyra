import { graphql } from "../../../@generated/gql";

export const UsersManagementQuery = graphql(`
  query UsersManagement {
    viewer {
      id
    }
    libraries {
      id
      name
      createdAt
    }
    users {
      id
      ...UserCard
    }
  }
`);

export const CreateUserInviteMutation = graphql(`
  mutation CreateUserInvite($username: String!, $permissions: Int!, $libraryIds: [String!]!) {
    createUserInvite(username: $username, permissions: $permissions, libraryIds: $libraryIds) {
      ...UserCard
    }
  }
`);

export const UpdateUserMutation = graphql(`
  mutation UpdateUser($userId: String!, $username: String!, $permissions: Int!, $libraryIds: [String!]!) {
    updateUser(userId: $userId, username: $username, permissions: $permissions, libraryIds: $libraryIds) {
      ...UserCard
    }
  }
`);

export const ResetUserInviteMutation = graphql(`
  mutation ResetUserInvite($userId: String!) {
    resetUserInvite(userId: $userId) {
      ...UserCard
    }
  }
`);

export const DeleteUserMutation = graphql(`
  mutation DeleteUser($userId: String!) {
    deleteUser(userId: $userId)
  }
`);
