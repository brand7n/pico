<?php

declare(strict_types=1);

namespace Remodulate\Pico;

/**
 * Tagged value for heterogeneous storage.
 *
 * Wraps a value with a type tag for runtime type checking.
 * Used where compile-time type information is insufficient
 * (e.g., parser semantic value stacks).
 *
 * LLVM layout: { i64, i64 } — 16 bytes (tag + bits).
 */
class Value
{
    public const NULL   = 0;
    public const INT    = 1;
    public const BOOL   = 2;
    public const FLOAT  = 3;
    public const STRING = 4;
    public const OBJECT = 5;

    private function __construct(
        private readonly int $tag,
        private readonly int $intVal,
        private readonly bool $boolVal,
        private readonly float $floatVal,
        private readonly ?PicoString $strVal,
        private readonly ?object $objVal,
    ) {}

    public static function none(): self
    {
        return new self(self::NULL, 0, false, 0.0, null, null);
    }

    public static function fromInt(int $v): self
    {
        return new self(self::INT, $v, false, 0.0, null, null);
    }

    public static function fromBool(bool $v): self
    {
        return new self(self::BOOL, 0, $v, 0.0, null, null);
    }

    public static function fromFloat(float $v): self
    {
        return new self(self::FLOAT, 0, false, $v, null, null);
    }

    public static function fromString(PicoString $v): self
    {
        return new self(self::STRING, 0, false, 0.0, $v, null);
    }

    public static function fromObject(object $v): self
    {
        return new self(self::OBJECT, 0, false, 0.0, null, $v);
    }

    public function tag(): int
    {
        return $this->tag;
    }

    public function isNull(): bool
    {
        return $this->tag === self::NULL;
    }

    public function asInt(): int
    {
        if ($this->tag !== self::INT) {
            throw new \RuntimeException(
                'Value: expected ' . self::tagName(self::INT) . ', got ' . self::tagName($this->tag)
            );
        }
        return $this->intVal;
    }

    public function asBool(): bool
    {
        if ($this->tag !== self::BOOL) {
            throw new \RuntimeException(
                'Value: expected ' . self::tagName(self::BOOL) . ', got ' . self::tagName($this->tag)
            );
        }
        return $this->boolVal;
    }

    public function asFloat(): float
    {
        if ($this->tag !== self::FLOAT) {
            throw new \RuntimeException(
                'Value: expected ' . self::tagName(self::FLOAT) . ', got ' . self::tagName($this->tag)
            );
        }
        return $this->floatVal;
    }

    public function asString(): PicoString
    {
        if ($this->tag !== self::STRING || $this->strVal === null) {
            throw new \RuntimeException(
                'Value: expected ' . self::tagName(self::STRING) . ', got ' . self::tagName($this->tag)
            );
        }
        return $this->strVal;
    }

    public function asObject(): object
    {
        if ($this->tag !== self::OBJECT || $this->objVal === null) {
            throw new \RuntimeException(
                'Value: expected ' . self::tagName(self::OBJECT) . ', got ' . self::tagName($this->tag)
            );
        }
        return $this->objVal;
    }

    private static function tagName(int $tag): string
    {
        return match ($tag) {
            self::NULL => 'NULL',
            self::INT => 'INT',
            self::BOOL => 'BOOL',
            self::FLOAT => 'FLOAT',
            self::STRING => 'STRING',
            self::OBJECT => 'OBJECT',
            default => "UNKNOWN({$tag})",
        };
    }
}
