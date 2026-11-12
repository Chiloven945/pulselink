package top.chiloven.pulselink.core

import kotlin.math.round

/**
 * A pressure reading in hundredths of a kilopascal (0.01 kPa).
 */
@JvmInline
public value class Pressure private constructor(
    public val centiKpa: Int
) : Comparable<Pressure> {

    override fun compareTo(other: Pressure): Int =
        centiKpa.compareTo(other.centiKpa)

    /** Returns the pressure in kilopascals. */
    public fun asKpa(): Float =
        centiKpa / 100.0f

    public companion object {

        /** Creates a pressure from hundredths of a kilopascal. */
        public fun fromCentiKpa(value: Int): Pressure =
            Pressure(value)

        /** Creates a pressure from kilopascals, rounded to 0.01 kPa. */
        public fun fromKpa(value: Float): Pressure? =
            when {
                !value.isFinite() -> null
                else -> fromScaled(value.toDouble())
            }

        /** Creates a pressure from kilopascals, rounded to 0.01 kPa. */
        public fun fromKpa(value: Double): Pressure? =
            when {
                !value.isFinite() -> null
                else -> fromScaled(value)
            }

        private fun fromScaled(value: Double): Pressure? =
            round(value * 100.0).let {
                when {
                    it < Int.MIN_VALUE.toDouble() || it > Int.MAX_VALUE.toDouble() -> null
                    else -> Pressure(it.toInt())
                }
            }

    }

}
