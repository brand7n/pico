<?php

declare(strict_types=1);

namespace Remodulate\Pico;

/**
 * Immutable byte string.
 *
 * All operations that modify content return a new PicoString.
 * Supports bracket syntax for character access ($s[0] returns int byte value).
 */
class PicoString implements \ArrayAccess
{
    public function __construct(
        private readonly string $data,
    ) {}

    public function length(): int
    {
        return strlen($this->data);
    }

    /**
     * Returns the byte value (0-255) at the given index.
     */
    public function charAt(int $index): int
    {
        if ($index < 0 || $index >= strlen($this->data)) {
            throw new \RuntimeException("PicoString::charAt({$index}) — index out of bounds (length={$this->length()})");
        }
        return ord($this->data[$index]);
    }

    public function substring(int $start, int $length = -1): self
    {
        if ($length < 0) {
            return new self(substr($this->data, $start));
        }
        return new self(substr($this->data, $start, $length));
    }

    public function indexOf(self $needle, int $offset = 0): int
    {
        $pos = strpos($this->data, $needle->data, $offset);
        return $pos === false ? -1 : $pos;
    }

    public function startsWith(self $prefix): bool
    {
        return str_starts_with($this->data, $prefix->data);
    }

    public function endsWith(self $suffix): bool
    {
        return str_ends_with($this->data, $suffix->data);
    }

    public function contains(self $needle): bool
    {
        return str_contains($this->data, $needle->data);
    }

    public function concat(self $other): self
    {
        return new self($this->data . $other->data);
    }

    public function equals(self $other): bool
    {
        return $this->data === $other->data;
    }

    public function trim(): self
    {
        return new self(trim($this->data));
    }

    public function toLower(): self
    {
        return new self(strtolower($this->data));
    }

    public function toUpper(): self
    {
        return new self(strtoupper($this->data));
    }

    public function replace(self $search, self $replace): self
    {
        return new self(str_replace($search->data, $replace->data, $this->data));
    }

    /**
     * @return Collection<PicoString>
     */
    public function split(self $delimiter): Collection
    {
        $parts = explode($delimiter->data, $this->data);
        $result = new Collection();
        foreach ($parts as $part) {
            $result->push(new self($part));
        }
        return $result;
    }

    public function toInt(): int
    {
        return (int) $this->data;
    }

    /**
     * Get the raw PHP string (for interop with PHP stdlib during development).
     */
    public function raw(): string
    {
        return $this->data;
    }

    public function __toString(): string
    {
        return $this->data;
    }

    // ── ArrayAccess for bracket syntax ─────────────────────────────
    // $s[0] returns int byte value. Write access is not allowed (immutable).

    public function offsetExists(mixed $offset): bool
    {
        return is_int($offset) && $offset >= 0 && $offset < strlen($this->data);
    }

    /**
     * @return int byte value at offset
     */
    public function offsetGet(mixed $offset): mixed
    {
        return $this->charAt((int) $offset);
    }

    public function offsetSet(mixed $offset, mixed $value): void
    {
        throw new \RuntimeException('PicoString is immutable — bracket assignment not supported');
    }

    public function offsetUnset(mixed $offset): void
    {
        throw new \RuntimeException('PicoString is immutable — unset not supported');
    }
}
