# -*- coding: utf-8 -*-
# Generated by the protocol buffer compiler.  DO NOT EDIT!
# source: tendermint/crypto/keys.proto
"""Generated protocol buffer code."""
from google.protobuf import descriptor as _descriptor
from google.protobuf import descriptor_pool as _descriptor_pool
from google.protobuf import message as _message
from google.protobuf import reflection as _reflection
from google.protobuf import symbol_database as _symbol_database
# @@protoc_insertion_point(imports)

_sym_db = _symbol_database.Default()


from gogoproto import gogo_pb2 as gogoproto_dot_gogo__pb2


DESCRIPTOR = _descriptor_pool.Default().AddSerializedFile(b'\n\x1ctendermint/crypto/keys.proto\x12\x11tendermint.crypto\x1a\x14gogoproto/gogo.proto\"D\n\tPublicKey\x12\x11\n\x07\x65\x64\x32\x35\x35\x31\x39\x18\x01 \x01(\x0cH\x00\x12\x13\n\tsecp256k1\x18\x02 \x01(\x0cH\x00:\x08\xe8\xa1\x1f\x01\xe8\xa0\x1f\x01\x42\x05\n\x03sumB:Z8github.com/tendermint/tendermint/proto/tendermint/cryptob\x06proto3')



_PUBLICKEY = DESCRIPTOR.message_types_by_name['PublicKey']
PublicKey = _reflection.GeneratedProtocolMessageType('PublicKey', (_message.Message,), {
  'DESCRIPTOR' : _PUBLICKEY,
  '__module__' : 'tendermint.crypto.keys_pb2'
  # @@protoc_insertion_point(class_scope:tendermint.crypto.PublicKey)
  })
_sym_db.RegisterMessage(PublicKey)

if _descriptor._USE_C_DESCRIPTORS == False:

  DESCRIPTOR._options = None
  DESCRIPTOR._serialized_options = b'Z8github.com/tendermint/tendermint/proto/tendermint/crypto'
  _PUBLICKEY._options = None
  _PUBLICKEY._serialized_options = b'\350\241\037\001\350\240\037\001'
  _PUBLICKEY._serialized_start=73
  _PUBLICKEY._serialized_end=141
# @@protoc_insertion_point(module_scope)
