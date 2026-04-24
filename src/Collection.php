<?php

declare(strict_types=1);

namespace Remodulate\Pico;

/**
 * Unified ordered collection — serves as both sequential array and associative map.
 *
 * Implements ArrayAccess for bracket syntax ($c[0], $c['key'], $c[] = $v)
 * and Countable for count($c).
 *
 * Type safety is enforced at compile time via @template T. The compiler resolves
 * T from @var annotations at use sites and emits type-specific runtime calls.
 * At PHP runtime, values are stored untyped (this is the reference implementation
 * for testing; the Rust native implementation uses typed storage).
 *
 * @template T
 * @implements \ArrayAccess<int|string, T>
 */
class Collection implements \ArrayAccess, \Countable
{
    /** @var array<int|string, T> */
    private array $data = [];

    /** @var int next sequential integer key for push() */
    private int $nextIntKey = 0;

    // ── Sequential (array-like) access ─────────────────────────────

    /**
     * Append a value with the next sequential integer key.
     * @param T $val
     */
    public function push(mixed $val): void
    {
        $this->data[$this->nextIntKey++] = $val;
    }

    /**
     * Remove and return the last element.
     * @return T
     */
    public function pop(): mixed
    {
        if ($this->data === []) {
            throw new \RuntimeException('Collection::pop() on empty collection');
        }
        $val = array_pop($this->data);
        $this->nextIntKey = max(0, $this->nextIntKey - 1);
        return $val;
    }

    /**
     * Get value at integer index.
     * @return T
     */
    public function getAt(int $index): mixed
    {
        if (!isset($this->data[$index])) {
            throw new \RuntimeException("Collection::getAt({$index}) — index out of bounds");
        }
        return $this->data[$index];
    }

    /**
     * Set value at integer index.
     * @param T $val
     */
    public function setAt(int $index, mixed $val): void
    {
        $this->data[$index] = $val;
    }

    /**
     * Return the last element without removing it.
     * @return T
     */
    public function last(): mixed
    {
        if ($this->data === []) {
            throw new \RuntimeException('Collection::last() on empty collection');
        }
        return end($this->data);
    }

    /**
     * Return a sub-collection.
     * @return Collection<T>
     */
    public function slice(int $start, int $length = -1): self
    {
        /** @var self<T> $result */
        $result = new self();
        $values = array_values($this->data);
        $sliced = $length < 0
            ? array_slice($values, $start)
            : array_slice($values, $start, $length);
        foreach ($sliced as $v) {
            $result->push($v);
        }
        return $result;
    }

    // ── Associative (map-like) access ──────────────────────────────

    /**
     * Get value by string key.
     * @return T
     */
    public function get(string $key): mixed
    {
        if (!array_key_exists($key, $this->data)) {
            throw new \RuntimeException("Collection::get('{$key}') — key not found");
        }
        return $this->data[$key];
    }

    /**
     * Set value by string key.
     * @param T $val
     */
    public function set(string $key, mixed $val): void
    {
        $this->data[$key] = $val;
    }

    /**
     * Check if a string key exists.
     */
    public function has(string $key): bool
    {
        return array_key_exists($key, $this->data);
    }

    /**
     * Get the string key at a positional index.
     */
    public function keyAt(int $index): string
    {
        $keys = array_keys($this->data);
        if (!isset($keys[$index])) {
            throw new \RuntimeException("Collection::keyAt({$index}) — index out of bounds");
        }
        return (string) $keys[$index];
    }

    // ── Common ─────────────────────────────────────────────────────

    public function count(): int
    {
        return count($this->data);
    }

    /**
     * Check if integer index is within bounds.
     */
    public function validIndex(int $index): bool
    {
        return $index >= 0 && $index < $this->nextIntKey && isset($this->data[$index]);
    }

    /**
     * Search for a value, return its positional index or -1.
     * @param T $needle
     */
    public function indexOf(mixed $needle): int
    {
        $i = 0;
        foreach ($this->data as $v) {
            if ($v === $needle) {
                return $i;
            }
            $i++;
        }
        return -1;
    }

    /**
     * Check if collection contains a value.
     * @param T $needle
     */
    public function containsValue(mixed $needle): bool
    {
        return $this->indexOf($needle) !== -1;
    }

    // ── ArrayAccess implementation ─────────────────────────────────
    // These methods make bracket syntax work under regular PHP.
    // The compiler bypasses them and emits direct runtime calls.

    /**
     * @param int|string $offset
     */
    public function offsetExists(mixed $offset): bool
    {
        if (is_int($offset)) {
            return $this->validIndex($offset);
        }
        return $this->has((string) $offset);
    }

    /**
     * @param int|string $offset
     * @return T
     */
    public function offsetGet(mixed $offset): mixed
    {
        if (is_int($offset)) {
            return $this->getAt($offset);
        }
        return $this->get((string) $offset);
    }

    /**
     * @param int|string|null $offset null means push ($c[] = $v)
     * @param T $value
     */
    public function offsetSet(mixed $offset, mixed $value): void
    {
        if ($offset === null) {
            $this->push($value);
        } elseif (is_int($offset)) {
            $this->setAt($offset, $value);
        } else {
            $this->set((string) $offset, $value);
        }
    }

    /**
     * @param int|string $offset
     */
    public function offsetUnset(mixed $offset): void
    {
        unset($this->data[$offset]);
    }

    // ── Countable implementation ───────────────────────────────────
    // count() is already defined above; Countable just requires it.

    // ── Factories ──────────────────────────────────────────────────

    /**
     * Create a collection from a PHP array literal.
     *
     * Under regular PHP: copies elements from the array.
     * Under compilation: the array literal is never allocated; the compiler
     * inlines it as new Collection() + push() for each element.
     *
     * @param array<T> $items
     * @return self<T>
     */
    public static function from(array $items): self
    {
        /** @var self<T> $c */
        $c = new self();
        foreach ($items as $key => $value) {
            if (is_int($key)) {
                $c->push($value);
            } else {
                $c->set((string) $key, $value);
            }
        }
        return $c;
    }

    /**
     * Create a collection from variadic arguments (sequential keys only).
     *
     * @param T ...$values
     * @return self<T>
     */
    public static function of(mixed ...$values): self
    {
        /** @var self<T> $c */
        $c = new self();
        foreach ($values as $v) {
            $c->push($v);
        }
        return $c;
    }
}
