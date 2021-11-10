//  Copyright (c) Facebook, Inc. and its affiliates.

import Foundation

public enum BinaryDeserializerError: Error {
    case invalidInput(issue: String)
}

public class BinaryDeserializer: Deserializer {
    fileprivate let input: [UInt8]
    fileprivate var location: Int
    fileprivate var containerDepthBudget: Int64

    init(input: [UInt8], maxContainerDepth: Int64) {
        self.input = input
        location = 0
        containerDepthBudget = maxContainerDepth
    }

    private func readBytes(count: Int) throws -> [UInt8] {
        let newLocation = location + count
        if newLocation > input.count {
            throw BinaryDeserializerError.invalidInput(issue: "Input is too small")
        }
        let bytes = input[location ..< newLocation]
        location = newLocation
        return Array(bytes)
    }

    public func deserialize_len() throws -> Int64 {
        assertionFailure("Not implemented")
        return 0
    }

    public func deserialize_variant_index() throws -> Int {
        assertionFailure("Not implemented")
        return 0
    }

    public func deserialize_char() throws -> Character {
        throw BinaryDeserializerError.invalidInput(issue: "Not implemented: char deserialization")
    }

    public func deserialize_f32() throws -> Float {
        let num = try deserialize_u32()
        return Float(bitPattern: num)
    }

    public func deserialize_f64() throws -> Double {
        let num = try deserialize_u64()
        return Double(bitPattern: num)
    }

    public func increase_container_depth() throws {
        if containerDepthBudget == 0 {
            throw BinaryDeserializerError.invalidInput(issue: "Exceeded maximum container depth")
        }
        containerDepthBudget -= 1
    }

    public func decrease_container_depth() {
        containerDepthBudget += 1
    }

    public func deserialize_str() throws -> String {
        let len: Int64 = try deserialize_len()
        if len < 0 || len > Int.max {
            throw BinaryDeserializerError.invalidInput(issue: "Incorrect length value for Swift string")
        }
        let content = try readBytes(count: Int(len))
        return String(bytes: content, encoding: .utf8)!
    }

    public func deserialize_bytes() throws -> [UInt8] {
        let len: Int64 = try deserialize_len()
        if len < 0 || len > Int.max {
            throw BinaryDeserializerError.invalidInput(issue: "Incorrect length value for Swift array")
        }
        let content = try readBytes(count: Int(len))
        return content
    }

    public func deserialize_bool() throws -> Bool {
        let byte = try readBytes(count: 1)[0]
        // TODO: reject values > 1
        return byte != 0
    }

    public func deserialize_unit() throws -> Unit {
        return Unit()
    }

    public func deserialize_u8() throws -> UInt8 {
        let bytes = try readBytes(count: 1)
        return bytes[0]
    }

    public func deserialize_u16() throws -> UInt16 {
        let bytes = try readBytes(count: 2)
        var x = UInt16(bytes[0])
        x += UInt16(bytes[1]) << 8
        return x
    }

    public func deserialize_u32() throws -> UInt32 {
        let bytes = try readBytes(count: 4)
        var x = UInt32(bytes[0])
        x += UInt32(bytes[1]) << 8
        x += UInt32(bytes[2]) << 16
        x += UInt32(bytes[3]) << 24
        return x
    }

    public func deserialize_u64() throws -> UInt64 {
        let bytes = try readBytes(count: 8)
        var x = UInt64(bytes[0])
        x += UInt64(bytes[1]) << 8
        x += UInt64(bytes[2]) << 16
        x += UInt64(bytes[3]) << 24
        x += UInt64(bytes[4]) << 32
        x += UInt64(bytes[5]) << 40
        x += UInt64(bytes[6]) << 48
        x += UInt64(bytes[7]) << 56
        return x
    }

    public func deserialize_u128() throws -> BigInt8 {
        let signed: BigInt8 = try deserialize_i128()
        if signed >= 0 {
            return signed
        } else {
            return signed + (BigInt8(1) << 128)
        }
    }

    public func deserialize_i8() throws -> Int8 {
        return Int8(bitPattern: try deserialize_u8())
    }

    public func deserialize_i16() throws -> Int16 {
        return Int16(bitPattern: try deserialize_u16())
    }

    public func deserialize_i32() throws -> Int32 {
        return Int32(bitPattern: try deserialize_u32())
    }

    public func deserialize_i64() throws -> Int64 {
        return Int64(bitPattern: try deserialize_u64())
    }

    public func deserialize_i128() throws -> BigInt8 {
        let content = try readBytes(count: 16)
        return BigInt8(content)
    }

    public func deserialize_option_tag() throws -> Bool {
        let value = try deserialize_u8()
        switch value {
        case 0: return false
        case 1: return true
        default: throw BinaryDeserializerError.invalidInput(issue: "Incorrect value for Option tag: \(value)")
        }
    }

    public func get_buffer_offset() -> Int {
        return location
    }

    public func check_that_key_slices_are_increasing(key1 _: Slice, key2 _: Slice) throws {
        assertionFailure("Not implemented")
    }
}
