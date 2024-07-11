import Foundation
import RegexBuilder

enum IntervalParsingError: Error, Equatable {
    case duplicatedUnit(Substring)
    case invalidFormat
    case invalidUnit(Substring)
}

extension IntervalParsingError: CustomStringConvertible {
    public var description: String {
        switch self {
        case .invalidUnit(let unit):
            return "Passed unit is not supported: \(unit)"
        case .duplicatedUnit(let unit):
            return "Passed unit is appears more than once: \(unit)"
        case .invalidFormat:
            return "Invalid time string format"
        }
    }
}

let kTimeStringRe = Regex {
    Capture {
        OneOrMore {
            CharacterClass(
                .anyOf("."),
                ("0"..."9")
            )
        }
    }
    Capture {
        OneOrMore {
            CharacterClass(
                ("a"..."z")
            )
        }
    }
}

extension TimeInterval {
    init(fromTimeString timeString: String) throws {
        let matches = timeString.matches(of: kTimeStringRe)
        if matches.isEmpty {
            throw IntervalParsingError.invalidFormat
        }
        var usedUnits: [Substring] = []
        var totalTimeInterval: TimeInterval = 0
        for match in matches {
            let (_, digits, unit) = match.output
            guard let value = Double(digits) else {
                throw IntervalParsingError.invalidFormat
            }
            if usedUnits.contains(unit) {
                throw IntervalParsingError.duplicatedUnit(unit)
            }
            usedUnits.append(unit)
            totalTimeInterval += try TimeInterval.unitInterval(unit, value)
        }
        self = totalTimeInterval
    }

    static func unitInterval(_ unit: Substring, _ value: TimeInterval) throws -> TimeInterval {
        switch unit {
        case "d":
            return value * 24 * 60 * 60
        case "h":
            return value * 60 * 60
        case "m":
            return value * 60
        case "s":
            return value
        case "ms":
            return value / 1000

        default:
            throw IntervalParsingError.invalidUnit(unit)
        }
    }
}
