// Copyright 2018 Kodebox, Inc.
// This file is part of CodeChain.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use ccrypto::Blake;
use ckey::{Address, Public, Signature};
use heapsize::HeapSizeOf;
use primitives::{Bytes, H160, H256};
use rlp::{Decodable, DecoderError, Encodable, RlpStream, UntrustedRlp};

use crate::transaction::Transaction;
use crate::ShardId;

const ASSET_TRANSACTION: u8 = 1;
const PAYMENT: u8 = 2;
const SET_REGULAR_KEY: u8 = 3;
const CREATE_SHARD: u8 = 4;
const SET_SHARD_OWNERS: u8 = 5;
const SET_SHARD_USERS: u8 = 6;
const WRAP_CCC: u8 = 7;
const STORE: u8 = 8;
const REMOVE: u8 = 9;
const CUSTOM: u8 = 0xFF;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    AssetTransaction {
        transaction: Transaction,
        approvals: Vec<Signature>,
    },
    Payment {
        receiver: Address,
        /// Transferred amount.
        amount: u64,
    },
    SetRegularKey {
        key: Public,
    },
    CreateShard,
    SetShardOwners {
        shard_id: ShardId,
        owners: Vec<Address>,
    },
    SetShardUsers {
        shard_id: ShardId,
        users: Vec<Address>,
    },
    WrapCCC {
        shard_id: ShardId,
        lock_script_hash: H160,
        parameters: Vec<Bytes>,
        amount: u64,
    },
    Custom {
        handler_id: u64,
        bytes: Bytes,
    },
    Store {
        content: String,
        certifier: Address,
        signature: Signature,
    },
    Remove {
        hash: H256,
        signature: Signature,
    },
}

impl Action {
    pub fn hash(&self) -> H256 {
        let rlp = self.rlp_bytes();
        Blake::blake(rlp)
    }
}

impl HeapSizeOf for Action {
    fn heap_size_of_children(&self) -> usize {
        match self {
            Action::AssetTransaction {
                transaction,
                approvals,
            } => transaction.heap_size_of_children() + approvals.heap_size_of_children(),
            Action::SetShardOwners {
                owners,
                ..
            } => owners.heap_size_of_children(),
            Action::SetShardUsers {
                users,
                ..
            } => users.heap_size_of_children(),
            Action::WrapCCC {
                parameters,
                ..
            } => parameters.heap_size_of_children(),
            _ => 0,
        }
    }
}

impl Encodable for Action {
    fn rlp_append(&self, s: &mut RlpStream) {
        match self {
            Action::AssetTransaction {
                transaction,
                approvals,
            } => {
                s.begin_list(3);
                s.append(&ASSET_TRANSACTION);
                s.append(transaction);
                s.append_list(approvals);
            }
            Action::Payment {
                receiver,
                amount,
            } => {
                s.begin_list(3);
                s.append(&PAYMENT);
                s.append(receiver);
                s.append(amount);
            }
            Action::SetRegularKey {
                key,
            } => {
                s.begin_list(2);
                s.append(&SET_REGULAR_KEY);
                s.append(key);
            }
            Action::CreateShard => {
                s.begin_list(1);
                s.append(&CREATE_SHARD);
            }
            Action::SetShardOwners {
                shard_id,
                owners,
            } => {
                s.begin_list(3);
                s.append(&SET_SHARD_OWNERS);
                s.append(shard_id);
                s.append_list(owners);
            }
            Action::SetShardUsers {
                shard_id,
                users,
            } => {
                s.begin_list(3);
                s.append(&SET_SHARD_USERS);
                s.append(shard_id);
                s.append_list(users);
            }
            Action::WrapCCC {
                shard_id,
                lock_script_hash,
                parameters,
                amount,
            } => {
                s.begin_list(5);
                s.append(&WRAP_CCC);
                s.append(shard_id);
                s.append(lock_script_hash);
                s.append(parameters);
                s.append(amount);
            }
            Action::Store {
                content,
                certifier,
                signature,
            } => {
                s.begin_list(4);
                s.append(&STORE);
                s.append(content);
                s.append(certifier);
                s.append(signature);
            }
            Action::Remove {
                hash,
                signature,
            } => {
                s.begin_list(3);
                s.append(&REMOVE);
                s.append(hash);
                s.append(signature);
            }
            Action::Custom {
                handler_id,
                bytes,
            } => {
                s.begin_list(3);
                s.append(&CUSTOM);
                s.append(handler_id);
                s.append(bytes);
            }
        }
    }
}

impl Decodable for Action {
    fn decode(rlp: &UntrustedRlp) -> Result<Self, DecoderError> {
        match rlp.val_at(0)? {
            ASSET_TRANSACTION => {
                if rlp.item_count()? != 3 {
                    return Err(DecoderError::RlpIncorrectListLen)
                }
                Ok(Action::AssetTransaction {
                    transaction: rlp.val_at(1)?,
                    approvals: rlp.list_at(2)?,
                })
            }
            PAYMENT => {
                if rlp.item_count()? != 3 {
                    return Err(DecoderError::RlpIncorrectListLen)
                }
                Ok(Action::Payment {
                    receiver: rlp.val_at(1)?,
                    amount: rlp.val_at(2)?,
                })
            }
            SET_REGULAR_KEY => {
                if rlp.item_count()? != 2 {
                    return Err(DecoderError::RlpIncorrectListLen)
                }
                Ok(Action::SetRegularKey {
                    key: rlp.val_at(1)?,
                })
            }
            CREATE_SHARD => {
                if rlp.item_count()? != 1 {
                    return Err(DecoderError::RlpIncorrectListLen)
                }
                Ok(Action::CreateShard)
            }
            SET_SHARD_OWNERS => {
                if rlp.item_count()? != 3 {
                    return Err(DecoderError::RlpIncorrectListLen)
                }
                Ok(Action::SetShardOwners {
                    shard_id: rlp.val_at(1)?,
                    owners: rlp.list_at(2)?,
                })
            }
            SET_SHARD_USERS => {
                if rlp.item_count()? != 3 {
                    return Err(DecoderError::RlpIncorrectListLen)
                }
                Ok(Action::SetShardUsers {
                    shard_id: rlp.val_at(1)?,
                    users: rlp.list_at(2)?,
                })
            }
            WRAP_CCC => {
                if rlp.item_count()? != 5 {
                    return Err(DecoderError::RlpIncorrectListLen)
                }
                Ok(Action::WrapCCC {
                    shard_id: rlp.val_at(1)?,
                    lock_script_hash: rlp.val_at(2)?,
                    parameters: rlp.val_at(3)?,
                    amount: rlp.val_at(4)?,
                })
            }
            STORE => {
                if rlp.item_count()? != 4 {
                    return Err(DecoderError::RlpIncorrectListLen)
                }
                Ok(Action::Store {
                    content: rlp.val_at(1)?,
                    certifier: rlp.val_at(2)?,
                    signature: rlp.val_at(3)?,
                })
            }
            REMOVE => {
                if rlp.item_count()? != 3 {
                    return Err(DecoderError::RlpIncorrectListLen)
                }
                Ok(Action::Remove {
                    hash: rlp.val_at(1)?,
                    signature: rlp.val_at(2)?,
                })
            }
            CUSTOM => {
                if rlp.item_count()? != 3 {
                    return Err(DecoderError::RlpIncorrectListLen)
                }
                Ok(Action::Custom {
                    handler_id: rlp.val_at(1)?,
                    bytes: rlp.val_at(2)?,
                })
            }
            _ => Err(DecoderError::Custom("Unexpected action prefix")),
        }
    }
}

#[cfg(test)]
mod tests {
    use rlp::rlp_encode_and_decode_test;

    use super::*;

    #[test]
    fn encode_and_decode_set_shard_owners() {
        rlp_encode_and_decode_test!(Action::SetShardOwners {
            shard_id: 1,
            owners: vec![Address::random(), Address::random()],
        });
    }

    #[test]
    fn encode_and_decode_set_shard_users() {
        rlp_encode_and_decode_test!(Action::SetShardUsers {
            shard_id: 1,
            users: vec![Address::random(), Address::random()],
        });
    }

    #[test]
    fn encode_and_decode_store() {
        rlp_encode_and_decode_test!(Action::Store {
            content: "CodeChain".to_string(),
            certifier: Address::random(),
            signature: Signature::random(),
        });
    }

    #[test]
    fn encode_and_decode_remove() {
        rlp_encode_and_decode_test!(Action::Remove {
            hash: H256::random(),
            signature: Signature::random(),
        });
    }
}
